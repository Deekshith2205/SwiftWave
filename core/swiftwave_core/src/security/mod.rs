//! Security layer — Noise protocol handshake and AEAD session management.
//!
//! SwiftWave Share uses the **Noise_XX** handshake pattern:
//! - Mutual authentication (both sides authenticate).
//! - No pre-shared public keys required.
//! - Forward secrecy via ephemeral X25519.
//!
//! After the handshake, all data is protected with **ChaCha20-Poly1305**
//! (preferred) or **AES-256-GCM** (hardware-accelerated fallback).
//!
//! # TODO (Phase 2)
//! - Implement `NoiseSession` using the `snow` crate.
//! - Implement `AeadCipher` wrappers using `chacha20poly1305`.
//! - Add session resumption tokens to avoid repeated handshakes.
//! - Pin peer public keys after first successful exchange (TOFU).

use crate::Result;
use async_trait::async_trait;

/// Cipher suite selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherSuite {
    /// ChaCha20-Poly1305 (preferred — software-friendly).
    ChaCha20Poly1305,
    /// AES-256-GCM (preferred when AES-NI hardware is available).
    Aes256Gcm,
}

impl Default for CipherSuite {
    fn default() -> Self {
        // Default to ChaCha20 for portability.
        Self::ChaCha20Poly1305
    }
}

/// The result of a successful Noise handshake — an encrypted session.
///
/// Exposes encrypt/decrypt operations only; key material is encapsulated.
#[async_trait]
pub trait SecuritySession: Send + Sync {
    /// Encrypt `plaintext` in place, appending an authentication tag.
    ///
    /// Returns the ciphertext + tag as a new `Vec<u8>`.
    ///
    /// # TODO (Phase 2): zero-copy variant using `bytes::BytesMut`.
    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt and authenticate `ciphertext`, returning plaintext.
    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;

    /// The cipher suite negotiated during the handshake.
    fn cipher_suite(&self) -> CipherSuite;
}

/// The `SecurityHandshake` trait drives the Noise_XX exchange.
///
/// After `complete()` both sides hold a symmetric `SecuritySession`.
#[async_trait]
pub trait SecurityHandshake: Send + Sync {
    /// The concrete session type produced after a successful handshake.
    type Session: SecuritySession;

    /// Perform the handshake over the given byte-stream adapter.
    ///
    /// `io` is a bidirectional byte stream (QUIC stream, TCP stream, etc.)
    /// The trait is intentionally I/O-agnostic.
    ///
    /// # TODO (Phase 2): wire `snow` handshake state machine here.
    async fn complete(
        &self,
        local_static_key: &[u8; 32],
        remote_public_key_hint: Option<&[u8; 32]>,
    ) -> Result<Self::Session>;
}

// ---------------------------------------------------------------------------
// Stub implementation
// ---------------------------------------------------------------------------

/// A stub security session that performs **no encryption**.
///
/// **MUST NOT be used in production.**
pub struct StubSession {
    cipher: CipherSuite,
}

impl StubSession {
    /// Create a new stub session.
    pub fn new() -> Self {
        Self {
            cipher: CipherSuite::default(),
        }
    }
}

impl Default for StubSession {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecuritySession for StubSession {
    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // TODO (Phase 2): Replace with real ChaCha20-Poly1305 / AES-GCM.
        Ok(plaintext.to_vec())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        // TODO (Phase 2): Replace with real decryption + authentication.
        Ok(ciphertext.to_vec())
    }

    fn cipher_suite(&self) -> CipherSuite {
        self.cipher
    }
}
