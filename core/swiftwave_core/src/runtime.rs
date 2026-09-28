//! Application runtime ownership and lifecycle.
//!
//! The `SwiftWaveRuntime` is the single source of truth for the application state.
//! It encapsulates the Tokio runtime, the core configuration, and the device identity.
//!
//! The FFI layer holds a pointer to an instance of `SwiftWaveRuntime` and manages
//! its lifecycle safely.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

use crate::config::CoreConfig;
use crate::device::identity::DeviceIdentity;
use crate::device::storage::SecureStorage;
use crate::discovery::Discovery;
use crate::error::{Result, SwiftWaveError};

/// Represents the current lifecycle state of the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// Runtime object is created but not yet fully initialized.
    Created,
    /// Core services are initialized.
    Initialized,
    /// The runtime is shutting down or has shut down.
    Shutdown,
}

static RUNTIME_INIT_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Temporary Phase 1 In-Memory Storage
// ---------------------------------------------------------------------------

/// **PHASE 1 TEMPORARY**: An in-memory implementation of `SecureStorage`.
///
/// This does not persist keys across process restarts. It is solely to allow
/// the runtime to pass initialization checks without requiring full OS-level
/// keystore integration (Phase 2).
///
/// DEVELOPMENT / TESTING ONLY.
pub struct InMemoryMockStorage {
    cache: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryMockStorage {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }
}

impl SecureStorage for InMemoryMockStorage {
    fn save_secret(&self, key: &str, secret: &[u8]) -> Result<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| SwiftWaveError::Internal("MockStorage cache lock poisoned".to_string()))?;
        cache.insert(key.to_string(), secret.to_vec());
        Ok(())
    }

    fn load_secret(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let cache = self
            .cache
            .lock()
            .map_err(|_| SwiftWaveError::Internal("MockStorage cache lock poisoned".to_string()))?;
        Ok(cache.get(key).cloned())
    }

    fn delete_secret(&self, key: &str) -> Result<()> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| SwiftWaveError::Internal("MockStorage cache lock poisoned".to_string()))?;
        cache.remove(key);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Application Runtime
// ---------------------------------------------------------------------------

/// The core runtime owning the Tokio asynchronous execution context,
/// configuration, and application state.
pub struct SwiftWaveRuntime {
    pub state: RwLock<LifecycleState>,
    pub tokio_rt: Runtime,
    pub config: RwLock<Option<CoreConfig>>,
    pub identity: RwLock<Option<DeviceIdentity>>,
    pub discovery: RwLock<Option<Box<dyn crate::discovery::Discovery>>>,
    pub storage: Arc<dyn SecureStorage>,
    pub(crate) quic_server: RwLock<Option<quinn::Endpoint>>,
    pub(crate) actual_quic_port: RwLock<Option<u16>>,
    pub(crate) accept_task: RwLock<Option<tokio::task::JoinHandle<()>>>,
    pub(crate) incoming_peers: broadcast::Sender<crate::device::identity::PeerIdentity>,
}

impl SwiftWaveRuntime {
    /// Create a new, uninitialized runtime with a dedicated Tokio context
    /// and the default development in-memory storage.
    pub fn new() -> Result<Self> {
        Self::new_with_storage(Arc::new(InMemoryMockStorage::new()))
    }

    /// Create a new runtime with a specifically injected secure storage implementation.
    pub fn new_with_storage(storage: Arc<dyn SecureStorage>) -> Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2) // keep RAM usage low for mobile
            .thread_name("swiftwave-worker")
            .enable_all()
            .build()
            .map_err(|e| {
                SwiftWaveError::Internal(format!("Failed to build tokio runtime: {}", e))
            })?;

        Ok(Self {
            state: RwLock::new(LifecycleState::Created),
            tokio_rt: rt,
            config: RwLock::new(None),
            identity: RwLock::new(None),
            discovery: RwLock::new(None),
            storage,
            quic_server: RwLock::new(None),
            actual_quic_port: RwLock::new(None),
            accept_task: RwLock::new(None),
            incoming_peers: broadcast::channel(16).0,
        })
    }

    /// Initialize the runtime components (config, identity).
    pub fn initialize(&self) -> Result<()> {
        // RUNTIME_INIT_LOCK guarantees that concurrent SwiftWaveRuntime initialization paths
        // cannot simultaneously perform the empty-storage `load -> generate -> save` sequence.
        // It specifically protects identity generation during process startup across runtimes.
        let _init_guard = RUNTIME_INIT_LOCK.lock().unwrap();

        let mut state = self
            .state
            .write()
            .map_err(|_| SwiftWaveError::Internal("Runtime state lock poisoned".to_string()))?;
        if *state != LifecycleState::Created {
            return Err(SwiftWaveError::Internal(
                "Runtime is already initialized or shutting down".to_string(),
            ));
        }

        // We explicitly install the global rustls crypto provider here.
        // Quinn requires a crypto provider to build TLS configurations.
        // By installing it once during runtime initialization, we avoid
        // needing to inject the provider into every QUIC endpoint builder.
        // We ignore the result because another test/runtime might have already installed it.
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();

        // Initialize default configuration
        let config = CoreConfig::default();
        let device_name = "SwiftWave Device"; // TODO (Phase 2): pull from config or OS

        let identity = DeviceIdentity::load_or_generate(device_name, self.storage.clone())?;

        // Start QUIC Server Endpoint
        let bind_addr = "0.0.0.0:0".parse().unwrap();
        let endpoint = self
            .tokio_rt
            .block_on(crate::transport::quic::build_server_endpoint(bind_addr))?;
        let port = endpoint
            .local_addr()
            .map_err(|e| SwiftWaveError::Internal(format!("Failed to get QUIC local addr: {e}")))?
            .port();

        let endpoint_clone = endpoint.clone();
        let identity_clone = identity.clone();
        let incoming_tx = self.incoming_peers.clone();

        *self
            .quic_server
            .write()
            .map_err(|_| SwiftWaveError::Internal("quic_server lock poisoned".into()))? =
            Some(endpoint);
        *self
            .actual_quic_port
            .write()
            .map_err(|_| SwiftWaveError::Internal("actual_quic_port lock poisoned".into()))? =
            Some(port);

        *self
            .config
            .write()
            .map_err(|_| SwiftWaveError::Internal("Runtime config lock poisoned".to_string()))? =
            Some(config);
        *self.identity.write().map_err(|_| {
            SwiftWaveError::Internal("Runtime identity lock poisoned".to_string())
        })? = Some(identity);

        let accept_task_handle = self.tokio_rt.spawn(async move {
            while let Some(incoming) = endpoint_clone.accept().await {
                let id_clone = identity_clone.clone();
                let tx = incoming_tx.clone();
                tokio::spawn(async move {
                    if let Ok(conn) = incoming.await {
                        if let Ok((mut send, mut recv)) = conn.accept_bi().await {
                            let result = crate::transport::quic::perform_noise_handshake(
                                &mut send, &mut recv, false, &id_clone,
                            )
                            .await;
                            if let Ok(peer) = result {
                                let _ = tx.send(peer);
                            }
                        }
                    }
                });
            }
        });

        *self
            .accept_task
            .write()
            .map_err(|_| SwiftWaveError::Internal("accept_task lock poisoned".into()))? =
            Some(accept_task_handle);

        *state = LifecycleState::Initialized;

        Ok(())
    }

    /// Shutdown the runtime.
    pub fn shutdown(&self) -> Result<()> {
        let mut state = self
            .state
            .write()
            .map_err(|_| SwiftWaveError::Internal("Runtime state lock poisoned".to_string()))?;
        if *state == LifecycleState::Shutdown {
            return Ok(());
        }

        *state = LifecycleState::Shutdown;

        // Explicitly stop discovery before clearing other resources
        let _ = self.stop_discovery();

        // Abort the accept loop
        let accept_task = {
            let mut accept_task_guard = self
                .accept_task
                .write()
                .map_err(|_| SwiftWaveError::Internal("accept_task lock poisoned".to_string()))?;
            accept_task_guard.take()
        };

        if let Some(task) = accept_task {
            task.abort();
            let _ = self.tokio_rt.block_on(task);
        }

        // Close the QUIC Server Endpoint so it stops listening
        if let Ok(mut server) = self.quic_server.write() {
            if let Some(endpoint) = server.take() {
                endpoint.close(0_u32.into(), b"runtime shutdown");
                self.tokio_rt.block_on(endpoint.wait_idle());
            }
        }
        if let Ok(mut port) = self.actual_quic_port.write() {
            *port = None;
        }

        // Clear resources safely
        if let Ok(mut identity) = self.identity.write() {
            *identity = None;
        }
        if let Ok(mut config) = self.config.write() {
            *config = None;
        }

        // Note: The actual Tokio runtime is dropped when `SwiftWaveRuntime` is dropped by the FFI boundary,
        // which will gracefully cancel all pending tasks.
        Ok(())
    }

    /// Connect to a generic QUIC endpoint and authenticate its identity via Noise XX.
    /// This performs NO discovery-claim binding. It purely returns the authenticated peer.
    pub async fn connect(
        &self,
        addr: std::net::SocketAddr,
    ) -> Result<(crate::device::identity::PeerIdentity, quinn::Connection)> {
        let endpoint = {
            let quic_server = self
                .quic_server
                .read()
                .map_err(|_| SwiftWaveError::Internal("quic_server lock poisoned".to_string()))?;
            quic_server
                .as_ref()
                .ok_or_else(|| {
                    SwiftWaveError::Internal("QUIC endpoint not initialized".to_string())
                })?
                .clone()
        };

        let conn = endpoint
            .connect(addr, "swiftwave.local")
            .map_err(|e| SwiftWaveError::QuicConnection(e.to_string()))?
            .await
            .map_err(|e| SwiftWaveError::QuicConnection(e.to_string()))?;

        let (mut send, mut recv) = conn
            .open_bi()
            .await
            .map_err(|e| SwiftWaveError::QuicConnection(e.to_string()))?;

        let identity = {
            let id_lock = self
                .identity
                .read()
                .map_err(|_| SwiftWaveError::Internal("identity lock poisoned".to_string()))?;
            id_lock
                .as_ref()
                .ok_or_else(|| SwiftWaveError::Internal("Identity not loaded".to_string()))?
                .clone()
        };

        let peer =
            crate::transport::quic::perform_noise_handshake(&mut send, &mut recv, true, &identity)
                .await?;

        Ok((peer, conn))
    }

    /// Connect to a discovered peer and securely bind its authenticated Noise identity
    /// against the fingerprint it claimed during discovery.
    pub async fn connect_with_claim(
        &self,
        addr: std::net::SocketAddr,
        expected_fingerprint: &crate::device::identity::PublicKeyFingerprint,
    ) -> Result<(crate::device::identity::PeerIdentity, quinn::Connection)> {
        let (peer, conn) = self.connect(addr).await?;
        peer.verify_binding(expected_fingerprint)?;
        Ok((peer, conn))
    }

    /// Starts mDNS discovery on the local network using the actual QUIC port.
    pub fn start_discovery(
        &self,
        tx: tokio::sync::mpsc::Sender<crate::discovery::DiscoveryEvent>,
    ) -> Result<()> {
        let state = self
            .state
            .read()
            .map_err(|_| SwiftWaveError::Internal("Runtime state lock poisoned".to_string()))?;
        if *state != LifecycleState::Initialized {
            return Err(SwiftWaveError::Internal(
                "Runtime is not initialized".to_string(),
            ));
        }

        let mut discovery_guard = self
            .discovery
            .write()
            .map_err(|_| SwiftWaveError::Internal("Runtime discovery lock poisoned".to_string()))?;

        if discovery_guard.is_some() {
            return Ok(()); // Already started
        }

        let identity_guard = self
            .identity
            .read()
            .map_err(|_| SwiftWaveError::Internal("Runtime identity lock poisoned".to_string()))?;
        let identity = identity_guard
            .as_ref()
            .ok_or_else(|| SwiftWaveError::Internal("Identity not loaded".to_string()))?;

        let quic_port = self
            .actual_quic_port
            .read()
            .map_err(|_| SwiftWaveError::Internal("actual_quic_port lock poisoned".into()))?
            .ok_or_else(|| SwiftWaveError::Internal("QUIC port not available".into()))?;

        let mut mdns = crate::discovery::mdns::MdnsDiscovery::new(
            identity.fingerprint(),
            identity.display_name().to_string(),
            quic_port,
        );

        // mdns.start is async, block on it to ensure it fully starts before returning to FFI
        self.tokio_rt.block_on(mdns.start(tx))?;

        *discovery_guard = Some(Box::new(mdns));

        Ok(())
    }

    /// Stops mDNS discovery.
    pub fn stop_discovery(&self) -> Result<()> {
        let mut discovery_guard = self
            .discovery
            .write()
            .map_err(|_| SwiftWaveError::Internal("Runtime discovery lock poisoned".to_string()))?;

        if let Some(mut mdns) = discovery_guard.take() {
            self.tokio_rt.block_on(mdns.stop())?;
        }
        Ok(())
    }

    /// Subscribe to incoming Peer identities.
    pub fn subscribe_incoming_peers(
        &self,
    ) -> broadcast::Receiver<crate::device::identity::PeerIdentity> {
        self.incoming_peers.subscribe()
    }

    /// Get the actual QUIC port the runtime is listening on.
    pub fn actual_quic_port(&self) -> Option<u16> {
        self.actual_quic_port.read().unwrap().clone()
    }

    /// Returns true if the accept task is currently running.
    pub fn is_accept_task_active(&self) -> bool {
        self.accept_task.read().unwrap().is_some()
    }

    /// Returns true if the QUIC server endpoint is currently active.
    pub fn is_quic_server_active(&self) -> bool {
        self.quic_server.read().unwrap().is_some()
    }
}
