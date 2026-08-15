//! Device identity: X25519 keypair, persistent storage, and fingerprinting.
//!
//! # Security design
//! - The private key is generated once with a CSPRNG (`OsRng`) and persisted
//!   to disk as JSON. The file MUST be protected by OS-level permissions
//!   (mode 0600 on Unix; encrypted user profile on Windows).
//! - The public key fingerprint is a BLAKE3 hash of the raw 32-byte public key,
//!   encoded in Base58 for human readability.
//! - We use **static** X25519 keys (not ephemeral per-session) so that a peer
//!   can persist and recognise a trusted device across sessions.
//! - Ephemeral session keys for the Noise handshake are derived separately
//!   in the `security::handshake` module — they are never stored.

use serde::{Deserialize, Serialize};
use std::path::Path;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::error::{Result, SwiftWaveError};

/// Opaque, stable identifier for a device (Base58-encoded BLAKE3 fingerprint).
pub type DeviceId = String;

/// Serialisable representation of the device's long-term identity.
///
/// The `private_key_bytes` field is **secret** and must never leave the device.
#[derive(Serialize, Deserialize)]
struct PersistedIdentity {
    /// Raw 32-byte static private key (X25519), hex-encoded for JSON safety.
    private_key_hex: String,
    /// Human-readable display name chosen by the user.
    display_name: String,
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
}

impl DeviceIdentity {
    /// Generate a brand-new identity backed by a fresh CSPRNG keypair.
    pub fn generate(display_name: impl Into<String>) -> Self {
        // OsRng is a cryptographically secure random number generator backed
        // by the operating system (getrandom on Linux, BCryptGenRandom on
        // Windows). We do NOT use `rand::thread_rng()` here.
        use rand_core::OsRng;
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self {
            secret,
            public,
            display_name: display_name.into(),
        }
    }

    /// Load a persisted identity from disk, or generate a fresh one if the
    /// file does not exist.
    pub fn load_or_generate(path: &Path, display_name: impl Into<String>) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            let identity = Self::generate(display_name);
            identity.save(path)?;
            Ok(identity)
        }
    }

    /// Deserialise an identity from disk.
    fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path).map_err(SwiftWaveError::Io)?;
        let persisted: PersistedIdentity =
            serde_json::from_str(&data).map_err(SwiftWaveError::Serialisation)?;

        let key_bytes = hex::decode_to_array::<32>(&persisted.private_key_hex)
            .map_err(|_| SwiftWaveError::Identity("Corrupt private key hex".into()))?;

        let secret = StaticSecret::from(key_bytes);
        let public = PublicKey::from(&secret);
        Ok(Self {
            secret,
            public,
            display_name: persisted.display_name,
        })
    }

    /// Persist the identity to disk.
    ///
    /// Creates parent directories if they do not exist.
    ///
    /// # Security note
    /// The caller (platform adapter) is responsible for restricting file
    /// permissions to the owning user immediately after this call.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(SwiftWaveError::Io)?;
        }
        let persisted = PersistedIdentity {
            private_key_hex: hex_encode(self.secret.as_bytes()),
            display_name: self.display_name.clone(),
        };
        let json =
            serde_json::to_string_pretty(&persisted).map_err(SwiftWaveError::Serialisation)?;
        std::fs::write(path, json).map_err(SwiftWaveError::Io)?;
        Ok(())
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
    ///
    /// 32 bytes of BLAKE3 output encodes to ~44 Base58 characters.
    /// The fingerprint is stable as long as the keypair is unchanged.
    pub fn fingerprint(&self) -> DeviceId {
        let hash = blake3::hash(self.public.as_bytes());
        bs58::encode(hash.as_bytes()).into_string()
    }

    /// Return a short, user-displayable version of the fingerprint (first 8 chars).
    pub fn short_fingerprint(&self) -> String {
        self.fingerprint().chars().take(8).collect()
    }

    /// Return the display name of this device.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Set or update the display name.
    pub fn set_display_name(&mut self, name: impl Into<String>) {
        self.display_name = name.into();
    }
}

/// Helper: hex-encode a byte slice to a `String`.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Helper: decode exactly N hex chars into `[u8; N]`.
mod hex {
    pub fn decode_to_array<const N: usize>(s: &str) -> std::result::Result<[u8; N], ()> {
        if s.len() != N * 2 {
            return Err(());
        }
        let mut out = [0u8; N];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hi = hex_val(chunk[0]).ok_or(())?;
            let lo = hex_val(chunk[1]).ok_or(())?;
            out[i] = (hi << 4) | lo;
        }
        Ok(out)
    }

    fn hex_val(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn generate_produces_distinct_keypairs() {
        let a = DeviceIdentity::generate("A");
        let b = DeviceIdentity::generate("B");
        assert_ne!(a.public_key_bytes(), b.public_key_bytes());
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let id = DeviceIdentity::generate("Test");
        assert_eq!(id.fingerprint(), id.fingerprint());
    }

    #[test]
    fn fingerprint_changes_with_keypair() {
        let a = DeviceIdentity::generate("A");
        let b = DeviceIdentity::generate("B");
        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn short_fingerprint_is_8_chars() {
        let id = DeviceIdentity::generate("Test");
        assert_eq!(id.short_fingerprint().len(), 8);
    }

    #[test]
    fn round_trip_persist() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("identity.json");

        let original = DeviceIdentity::generate("Round-trip test");
        original.save(&path).unwrap();

        let loaded = DeviceIdentity::load(&path).unwrap();
        assert_eq!(loaded.public_key_bytes(), original.public_key_bytes());
        assert_eq!(loaded.display_name(), original.display_name());
        assert_eq!(loaded.fingerprint(), original.fingerprint());
    }

    #[test]
    fn load_or_generate_creates_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("id.json");
        assert!(!path.exists());

        let id = DeviceIdentity::load_or_generate(&path, "TestDevice").unwrap();
        assert!(path.exists());

        // Loading again must return the same identity.
        let id2 = DeviceIdentity::load_or_generate(&path, "Ignored").unwrap();
        assert_eq!(id.fingerprint(), id2.fingerprint());
    }
}
