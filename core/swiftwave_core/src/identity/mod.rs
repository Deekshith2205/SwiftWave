//! Device identity — keypair management and stable device identifiers.
//!
//! Each device generates a long-lived X25519 keypair on first run.
//! The public key doubles as the device's `DeviceId` after hashing.
//!
//! # TODO (Phase 2)
//! - Replace placeholder with real `x25519-dalek` keypair generation.
//! - Persist keypair to platform secure storage (Keystore / Keychain / DPAPI).
//! - Sign identity claims with the private key for peer verification.

use crate::{SwiftWaveError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stable, globally unique identifier derived from the device's public key.
///
/// In Phase 2 this will be `SHA-256(X25519_public_key)[..16]` encoded as UUID.
/// For now it wraps a random UUIDv4.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(Uuid);

impl DeviceId {
    /// Generate a new random `DeviceId` (placeholder — see TODO above).
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Return the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Human-readable device metadata announced during discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// The stable device identifier.
    pub id: DeviceId,
    /// User-visible device name (e.g. "Alice's Laptop").
    pub display_name: String,
    /// Application version string.
    pub app_version: String,
}

/// The `DeviceIdentity` trait abstracts key-pair storage and signing.
///
/// Implementors must guarantee that the private key **never** leaves the
/// secure enclave / keystore boundary.
#[async_trait::async_trait]
pub trait DeviceIdentity: Send + Sync {
    /// Return the device's stable `DeviceId`.
    fn device_id(&self) -> &DeviceId;

    /// Return public metadata safe to broadcast over the air.
    fn device_info(&self) -> &DeviceInfo;

    /// Export the raw X25519 public key bytes (32 bytes).
    ///
    /// Used during the Noise handshake.
    ///
    /// # TODO (Phase 2)
    /// Replace placeholder return with real key bytes from `x25519-dalek`.
    fn public_key_bytes(&self) -> [u8; 32];

    /// Sign arbitrary `data` with the device's private key.
    ///
    /// # TODO (Phase 2)
    /// Implement with `ed25519-dalek` or a Noise-compatible signing scheme.
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// ---------------------------------------------------------------------------
// Stub implementation — used in tests and until Phase 2 crypto lands.
// ---------------------------------------------------------------------------

/// A stub identity backed by a random UUID and zeroed key material.
///
/// **MUST NOT be used in production.** Exists solely for testing and
/// scaffolding purposes.
pub struct StubIdentity {
    info: DeviceInfo,
}

impl StubIdentity {
    /// Create a new stub identity with a random device ID.
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            info: DeviceInfo {
                id: DeviceId::generate(),
                display_name: display_name.into(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

#[async_trait::async_trait]
impl DeviceIdentity for StubIdentity {
    fn device_id(&self) -> &DeviceId {
        &self.info.id
    }

    fn device_info(&self) -> &DeviceInfo {
        &self.info
    }

    fn public_key_bytes(&self) -> [u8; 32] {
        // TODO (Phase 2): Return real X25519 public key.
        [0u8; 32]
    }

    async fn sign(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // TODO (Phase 2): Implement real Ed25519/X25519 signing.
        Err(SwiftWaveError::Identity(
            "StubIdentity does not support signing".into(),
        ))
    }
}
