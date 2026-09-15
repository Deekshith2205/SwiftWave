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
