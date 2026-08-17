//! Device identity: X25519 keypair, persistent storage, and fingerprinting.
//!
//! # Security design
//! - The private key is generated once with a CSPRNG (`OsRng`) and persisted
//!   using the `SecureStorage` trait. This ensures it relies on the OS keystore.
//! - The public key fingerprint is a BLAKE3 hash of the raw 32-byte public key,
//!   encoded in Base58 for human readability.
//! - We use **static** X25519 keys (not ephemeral per-session) so that a peer
//!   can persist and recognise a trusted device across sessions.
//! - Ephemeral session keys for the Noise handshake are derived separately
//!   in the `security::session` module — they are never stored.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::device::storage::{SecureStorage, IDENTITY_SECRET_KEY};
use crate::error::{Result, SwiftWaveError};

/// A strongly-typed wrapper around the Base58-encoded BLAKE3 fingerprint of a public key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKeyFingerprint(pub String);

impl PublicKeyFingerprint {
    /// Return a short, user-displayable version of the fingerprint (first 8 chars).
    pub fn short(&self) -> String {
        self.0.chars().take(8).collect()
    }
}

/// A device's persistent cryptographic identity.
///
/// Holds the long-term X25519 static keypair used for:
/// 1. Noise_XX handshake (static public key exchange).
/// 2. Peer fingerprint / trust management.
#[derive(Clone)]
pub struct DeviceIdentity {
    /// Long-term static secret key.
    ///
    /// # Security note
    /// Never log, transmit, or expose this value. It is zeroised when dropped
    /// because `StaticSecret` implements `ZeroizeOnDrop`.
    secret: StaticSecret,
    /// Corresponding public key.
    public: PublicKey,
    /// User-visible display name (e.g. "Alice's Laptop").
    display_name: String,
    /// Secure storage backend.
    storage: Arc<dyn SecureStorage>,
}

impl DeviceIdentity {
    /// Generate a brand-new identity backed by a fresh CSPRNG keypair.
    pub fn generate(display_name: impl Into<String>, storage: Arc<dyn SecureStorage>) -> Result<Self> {
        // OsRng is a cryptographically secure random number generator backed
        // by the operating system.
        use rand_core::OsRng;
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        
        let identity = Self {
            secret,
            public,
            display_name: display_name.into(),
            storage,
        };
        identity.save()?;
        Ok(identity)
    }

    /// Load a persisted identity from storage, or generate a fresh one if it doesn't exist.
    pub fn load_or_generate(display_name: impl Into<String>, storage: Arc<dyn SecureStorage>) -> Result<Self> {
        if let Some(secret_bytes) = storage.load_secret(IDENTITY_SECRET_KEY)? {
            let key_array: [u8; 32] = secret_bytes.try_into()
                .map_err(|_| SwiftWaveError::Identity("Corrupted secret key length".into()))?;
            let secret = StaticSecret::from(key_array);
            let public = PublicKey::from(&secret);
            
            return Ok(Self {
                secret,
                public,
                display_name: display_name.into(),
                storage,
            });
        }
        
        Self::generate(display_name, storage)
    }

    /// Persist the identity to the secure storage abstraction.
    pub fn save(&self) -> Result<()> {
        self.storage.save_secret(IDENTITY_SECRET_KEY, self.secret.as_bytes())
    }

    /// Return the raw 32-byte public key.
    pub fn public_key_bytes(&self) -> &[u8; 32] {
        self.public.as_bytes()
    }

    /// Return the raw 32-byte static secret key.
    ///
    /// # Security note
    /// Only expose this to the Noise handshake engine.
    pub fn secret_key_bytes(&self) -> &[u8; 32] {
        self.secret.as_bytes()
    }

    /// Return a stable, human-readable fingerprint of the public key.
    ///
    /// Algorithm: `Base58( BLAKE3( public_key_bytes ) )`.
    pub fn fingerprint(&self) -> PublicKeyFingerprint {
        let hash = blake3::hash(self.public.as_bytes());
        PublicKeyFingerprint(bs58::encode(hash.as_bytes()).into_string())
    }

    /// Return the display name of this device.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Generate a safe QR code payload representing this device's public identity.
    /// Format: `swiftwave://id/<fingerprint>?name=<url_encoded_name>`
    pub fn qr_code_payload(&self) -> String {
        let fp = self.fingerprint().0;
        let encoded_name = urlencoding::encode(&self.display_name);
        format!("swiftwave://id/{}?name={}", fp, encoded_name)
    }
}

/// Information about a remote peer's public identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerIdentity {
    /// The peer's stable fingerprint.
    pub fingerprint: PublicKeyFingerprint,
    /// The peer's advertised display name.
    pub display_name: String,
    /// The peer's raw static public key.
    pub public_key: [u8; 32],
}

impl PeerIdentity {
    /// Reconstruct from raw public key bytes.
    pub fn from_public_key(public_key: [u8; 32], display_name: impl Into<String>) -> Self {
        let hash = blake3::hash(&public_key);
        let fp = PublicKeyFingerprint(bs58::encode(hash.as_bytes()).into_string());
        Self {
            fingerprint: fp,
            display_name: display_name.into(),
            public_key,
        }
    }
}
