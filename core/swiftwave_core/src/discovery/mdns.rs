//! mDNS/DNS-SD discovery implementation for SwiftWave.
//!
//! # Security
//! Discovery metadata is UNTRUSTED. The advertised fingerprint is merely an identity
//! assertion. It must be cryptographically authenticated by Noise XX at the transport layer.

use std::collections::HashMap;

use crate::device::identity::PublicKeyFingerprint;

/// The canonical DNS-SD service type for SwiftWave over QUIC (UDP).
pub const SWIFTWAVE_SERVICE_TYPE: &str = "_swiftwave._udp.local.";

/// Maximum length for a display name in TXT records (arbitrary safe limit to prevent unbounded allocations).
pub const MAX_DISPLAY_NAME_LEN: usize = 63;

/// Supported SwiftWave discovery protocol version.
pub const SUPPORTED_DISCOVERY_VERSION: &str = "1";

/// Structured representation of parsed SwiftWave TXT metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwiftWaveTxtRecord {
    pub fingerprint: PublicKeyFingerprint,
    pub display_name: String,
}

impl SwiftWaveTxtRecord {
    /// Parses a raw TXT property map into a structured `SwiftWaveTxtRecord`.
    ///
    /// # Failures
    /// Returns `None` if the record is malformed, missing required keys,
    /// advertises an unsupported version, or violates size constraints.
    /// This function will *never* panic on network-controlled data.
    pub fn parse(properties: &HashMap<String, String>) -> Option<Self> {
        // 1. Version check
        let version = properties.get("v")?;
        if version != SUPPORTED_DISCOVERY_VERSION {
            return None;
        }

        // 2. Fingerprint check
        let fp_str = properties.get("fp")?;
        if fp_str.is_empty() {
            return None;
        }
        // Basic length bounds for a valid Base58 Blake3 hash
        if fp_str.len() < 32 || fp_str.len() > 64 {
            return None;
        }
        let fingerprint = PublicKeyFingerprint(fp_str.clone());

        // 3. Display name check
        let name_str = properties.get("name")?;
        if name_str.is_empty() {
            return None;
        }
        
        let mut display_name = name_str.clone();
        
        // DNS-SD limits total TXT to 65535, but individual key-value pairs are up to 255 bytes.
        // Truncate cleanly on UTF-8 char boundary if it somehow exceeds our safe application limit.
        if display_name.len() > MAX_DISPLAY_NAME_LEN {
            if let Some(idx) = display_name.char_indices().map(|(i, _)| i).find(|&i| i > MAX_DISPLAY_NAME_LEN) {
                display_name.truncate(idx);
            } else {
                display_name.truncate(MAX_DISPLAY_NAME_LEN);
            }
        }

        Some(Self {
            fingerprint,
            display_name,
        })
    }

    /// Serializes this record into a property map for mDNS broadcast.
    pub fn to_properties(&self) -> HashMap<String, String> {
        let mut props = HashMap::new();
        props.insert("v".to_string(), SUPPORTED_DISCOVERY_VERSION.to_string());
        props.insert("fp".to_string(), self.fingerprint.0.clone());
        props.insert("name".to_string(), self.display_name.clone());
        props
    }
}

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::discovery::{DiscoveredPeer, Discovery, DiscoveryEvent, DiscoveryMedium};
use crate::error::{Result, SwiftWaveError};

/// The local mDNS discovery orchestrator.
pub struct MdnsDiscovery {
    /// Local identity.
    identity_fingerprint: PublicKeyFingerprint,
    /// Local display name.
    display_name: String,
    /// QUIC port this device is listening on.
    quic_port: u16,
    
    /// Handle to the underlying mDNS daemon.
    daemon: Option<ServiceDaemon>,
    /// Handle to the background Tokio task processing mDNS events.
    task_handle: Option<JoinHandle<()>>,
}

impl MdnsDiscovery {
    /// Create a new mDNS discovery orchestrator.
    pub fn new(fingerprint: PublicKeyFingerprint, display_name: String, quic_port: u16) -> Self {
        Self {
            identity_fingerprint: fingerprint,
            display_name,
            quic_port,
            daemon: None,
            task_handle: None,
        }
    }
}

#[async_trait::async_trait]
impl Discovery for MdnsDiscovery {
    async fn start(&mut self, tx: mpsc::Sender<DiscoveryEvent>) -> Result<()> {
        if self.daemon.is_some() {
            return Ok(()); // Already running
        }

        let daemon = ServiceDaemon::new().map_err(|e| SwiftWaveError::Platform(format!("Failed to start mDNS daemon: {}", e)))?;
        
        let local_fp = self.identity_fingerprint.clone();
        let instance_name = local_fp.short(); // Bounded deterministic instance name

        let txt_record = SwiftWaveTxtRecord {
            fingerprint: local_fp.clone(),
            display_name: self.display_name.clone(),
        };

        // Construct host name (mdns-sd requires it to end with .local.)
        let host_name = format!("{}.local.", instance_name);

        let service_info = ServiceInfo::new(
            SWIFTWAVE_SERVICE_TYPE,
            &instance_name,
            &host_name,
            "", // no explicit IP array, daemon resolves it
            self.quic_port,
            txt_record.to_properties(),
        ).map_err(|e| SwiftWaveError::Platform(format!("Invalid mDNS service info: {}", e)))?;

        daemon.register(service_info).map_err(|e| SwiftWaveError::Platform(format!("Failed to register mDNS service: {}", e)))?;

        let receiver = daemon.browse(SWIFTWAVE_SERVICE_TYPE).map_err(|e| SwiftWaveError::Platform(format!("Failed to browse mDNS: {}", e)))?;

        let task_handle = tokio::spawn(async move {
            let mut registry: HashMap<PublicKeyFingerprint, DiscoveredPeer> = HashMap::new();

            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        // Parse UNTRUSTED TXT records
                        let txt_map = info.get_properties().clone(); // get_properties returns &ServiceProperties
                        let mut txt_map_str = std::collections::HashMap::new();
                        for prop in txt_map.iter() {
                            txt_map_str.insert(prop.key().to_string(), prop.val_str().to_string());
                        }
                        
                        let parsed = match SwiftWaveTxtRecord::parse(&txt_map_str) {
                            Some(r) => r,
                            None => continue, // Malformed, ignore gracefully
                        };

                        // Filter self
                        if parsed.fingerprint == local_fp {
                            continue;
                        }

                        // Determine the endpoint (prefer IPv4 if both available for simplicity, or grab any)
                        let address = match info.get_addresses().iter().next() {
                            Some(ip) => {
                                // ip is usually a ScopedIp or IpAddr. We can format it to bypass strict type mismatch,
                                // but we need an IpAddr. We can use the Display impl and parse back to IpAddr.
                                // Actually, ScopedIp usually derefs to IpAddr or has .to_string() returning just the IP if no scope.
                                let ip_str = ip.to_string();
                                let ip_cleaned = ip_str.split('%').next().unwrap_or(&ip_str); // remove scope id for parsing if present
                                match ip_cleaned.parse::<std::net::IpAddr>() {
                                    Ok(parsed_ip) => SocketAddr::new(parsed_ip, info.get_port()),
                                    Err(_) => continue,
                                }
                            },
                            None => continue,
                        };

                        let now_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

                        let peer = DiscoveredPeer {
                            fingerprint: parsed.fingerprint.clone(),
                            display_name: parsed.display_name,
                            address,
                            medium: DiscoveryMedium::MdnsUdp,
                            rssi: None,
                            protocol_version: 1, // Parsed successfully, so v=1
                            last_seen: now_unix,
                        };

                        // Registry logic
                        if let Some(existing) = registry.get(&parsed.fingerprint) {
                            // Update if endpoint or metadata changed
                            if existing.address != peer.address || existing.display_name != peer.display_name {
                                registry.insert(parsed.fingerprint.clone(), peer.clone());
                                let _ = tx.send(DiscoveryEvent::PeerFound(peer)).await;
                            }
                        } else {
                            // First discovery or reappearance
                            registry.insert(parsed.fingerprint.clone(), peer.clone());
                            let _ = tx.send(DiscoveryEvent::PeerFound(peer)).await;
                        }
                    },
                    ServiceEvent::ServiceRemoved(_type, fullname) => {
                        // fullname is instance_name._swiftwave._udp.local.
                        // We can just iterate the registry and remove any that might match?
                        // Actually, fullname starts with instance_name. Our instance name is short_hash.
                        let instance_name = fullname.split('.').next().unwrap_or("");
                        
                        // Find matching peer (since instance_name is derived from short hash, but could collide).
                        // Best effort removal by prefix match if necessary, or just check all peers.
                        registry.retain(|fp, _peer| {
                            let match_instance = fp.short() == instance_name;
                            if match_instance {
                                let _ = tx.try_send(DiscoveryEvent::PeerLost(fp.clone()));
                                false // remove from registry
                            } else {
                                true
                            }
                        });
                    },
                    ServiceEvent::SearchStarted(_) | ServiceEvent::SearchStopped(_) | ServiceEvent::ServiceFound(_, _) => {
                        // Ignore, wait for ServiceResolved
                    },
                    _ => {
                        // Non-exhaustive enum fallback
                    }
                }
            }
        });

        self.daemon = Some(daemon);
        self.task_handle = Some(task_handle);

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(task) = self.task_handle.take() {
            task.abort();
        }
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.shutdown();
        }
        Ok(())
    }
}

