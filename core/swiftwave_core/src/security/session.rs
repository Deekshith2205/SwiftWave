//! Secure session abstraction over an established Noise TransportState.

use snow::TransportState;
use crate::error::{Result, SwiftWaveError};

/// A mutually authenticated and encrypted session with a peer.
///
/// Wraps a Noise `TransportState` to encrypt and decrypt application data
/// (e.g., chunk payloads or control messages) using the ephemeral shared
/// secret established during the Noise_XX handshake.
pub struct SecureSession {
    transport: TransportState,
}

impl SecureSession {
    /// Create a new secure session from an established Noise transport state.
    pub fn new(transport: TransportState) -> Self {
        Self { transport }
    }

    /// Encrypt a plaintext message into the given output buffer.
    ///
    /// Returns the length of the written ciphertext. The output buffer must
    /// have enough capacity for the plaintext plus a 16-byte MAC.
    pub fn encrypt_message(&mut self, plaintext: &[u8], ciphertext: &mut [u8]) -> Result<usize> {
        self.transport
            .write_message(plaintext, ciphertext)
            .map_err(|_| SwiftWaveError::EncryptionFailed)
    }

    /// Decrypt a ciphertext message into the given output buffer.
    ///
    /// Returns the length of the written plaintext.
    pub fn decrypt_message(&mut self, ciphertext: &[u8], plaintext: &mut [u8]) -> Result<usize> {
        self.transport
            .read_message(ciphertext, plaintext)
            .map_err(|_| SwiftWaveError::DecryptionFailed)
    }

    /// Rekey the transport state.
    ///
    /// Useful for enforcing forward secrecy over very long-lived connections.
    /// Both peers must agree on when to rekey.
    pub fn rekey(&mut self) {
        self.transport.rekey_outgoing();
        self.transport.rekey_incoming();
    }
}
