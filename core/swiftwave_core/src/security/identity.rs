//! Peer identity verification after Noise handshake.
//!
//! After the Noise XX handshake completes, we have the remote peer's static
//! public key. This module manages the trust store: a list of previously
//! trusted peer fingerprints persisted on disk.
//!
//! Trust levels:
//! - `Unknown`  — never seen before.
//! - `Pending`  — SAS has been shown, awaiting user confirmation.
//! - `Trusted`  — user has explicitly confirmed the SAS.
//! - `Blocked`  — user has explicitly denied/blocked this peer.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BLAKE3+Base58 fingerprint of a peer's static public key.
pub type PeerFingerprint = String;

/// Trust level for a peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// Never seen before.
    Unknown,
    /// SAS displayed, awaiting user confirmation.
    Pending,
    /// Explicitly trusted by the user.
    Trusted,
    /// Explicitly blocked by the user.
    Blocked,
}

/// A record in the trust store for a single peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRecord {
    /// Human-readable name of the peer (from their `DeviceIdentity`).
    pub display_name: String,
    /// Raw public key bytes (32 bytes, hex-encoded for JSON).
    pub public_key_hex: String,
    /// Trust level.
    pub trust: TrustLevel,
    /// Timestamp of last successful connection (Unix seconds).
    pub last_seen: Option<u64>,
}

/// In-memory + optionally persisted trust store.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrustStore {
    peers: HashMap<PeerFingerprint, PeerRecord>,
}

impl TrustStore {
    /// Look up the trust level for a fingerprint.
    pub fn trust_level(&self, fp: &str) -> TrustLevel {
        self.peers
            .get(fp)
            .map(|r| r.trust.clone())
            .unwrap_or(TrustLevel::Unknown)
    }

    /// Add or update a peer record.
    pub fn upsert(&mut self, fp: PeerFingerprint, record: PeerRecord) {
        self.peers.insert(fp, record);
    }

    /// Mark a peer as trusted.
    pub fn set_trusted(&mut self, fp: &str) {
        if let Some(r) = self.peers.get_mut(fp) {
            r.trust = TrustLevel::Trusted;
        }
    }

    /// Mark a peer as blocked.
    pub fn set_blocked(&mut self, fp: &str) {
        if let Some(r) = self.peers.get_mut(fp) {
            r.trust = TrustLevel::Blocked;
        }
    }

    /// Return the fingerprint of a public key (BLAKE3 of the raw 32 bytes, Base58).
    pub fn fingerprint_of(public_key_bytes: &[u8]) -> PeerFingerprint {
        let hash = blake3::hash(public_key_bytes);
        bs58::encode(hash.as_bytes()).into_string()
    }

    /// Serialise to JSON.
    pub fn to_json(&self) -> crate::error::Result<String> {
        serde_json::to_string_pretty(self).map_err(crate::error::SwiftWaveError::Serialisation)
    }

    /// Deserialise from JSON.
    pub fn from_json(json: &str) -> crate::error::Result<Self> {
        serde_json::from_str(json).map_err(crate::error::SwiftWaveError::Serialisation)
    }
}
