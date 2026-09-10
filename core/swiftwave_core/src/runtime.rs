//! Application runtime ownership and lifecycle.
//!
//! The `SwiftWaveRuntime` is the single source of truth for the application state.
//! It encapsulates the Tokio runtime, the core configuration, and the device identity.
//!
//! The FFI layer holds a pointer to an instance of `SwiftWaveRuntime` and manages
//! its lifecycle safely.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use tokio::runtime::Runtime;

use crate::config::CoreConfig;
use crate::device::identity::DeviceIdentity;
use crate::device::storage::SecureStorage;
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
        let mut cache = self.cache.lock().map_err(|_| SwiftWaveError::Internal("MockStorage cache lock poisoned".to_string()))?;
        cache.insert(key.to_string(), secret.to_vec());
        Ok(())
    }

    fn load_secret(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let cache = self.cache.lock().map_err(|_| SwiftWaveError::Internal("MockStorage cache lock poisoned".to_string()))?;
        Ok(cache.get(key).cloned())
    }

    fn delete_secret(&self, key: &str) -> Result<()> {
        let mut cache = self.cache.lock().map_err(|_| SwiftWaveError::Internal("MockStorage cache lock poisoned".to_string()))?;
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
    pub storage: Arc<dyn SecureStorage>,
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
            .map_err(|e| SwiftWaveError::Internal(format!("Failed to build tokio runtime: {}", e)))?;

        Ok(Self {
            state: RwLock::new(LifecycleState::Created),
            tokio_rt: rt,
            config: RwLock::new(None),
            identity: RwLock::new(None),
            storage,
        })
    }

    /// Initialize the runtime components (config, identity).
    pub fn initialize(&self) -> Result<()> {
        // RUNTIME_INIT_LOCK guarantees that concurrent SwiftWaveRuntime initialization paths
        // cannot simultaneously perform the empty-storage `load -> generate -> save` sequence.
        // It specifically protects identity generation during process startup across runtimes.
        let _init_guard = RUNTIME_INIT_LOCK.lock().unwrap();

        let mut state = self.state.write().map_err(|_| SwiftWaveError::Internal("Runtime state lock poisoned".to_string()))?;
        if *state != LifecycleState::Created {
            return Err(SwiftWaveError::Internal("Runtime is already initialized or shutting down".to_string()));
        }

        // Initialize default configuration
        let config = CoreConfig::default();
        let device_name = "SwiftWave Device"; // TODO (Phase 2): pull from config or OS

        let identity = DeviceIdentity::load_or_generate(device_name, self.storage.clone())?;

        *self.config.write().map_err(|_| SwiftWaveError::Internal("Runtime config lock poisoned".to_string()))? = Some(config);
        *self.identity.write().map_err(|_| SwiftWaveError::Internal("Runtime identity lock poisoned".to_string()))? = Some(identity);
        
        *state = LifecycleState::Initialized;

        Ok(())
    }

    /// Shutdown the runtime.
    pub fn shutdown(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|_| SwiftWaveError::Internal("Runtime state lock poisoned".to_string()))?;
        if *state == LifecycleState::Shutdown {
            return Ok(());
        }

        *state = LifecycleState::Shutdown;
        
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
}
