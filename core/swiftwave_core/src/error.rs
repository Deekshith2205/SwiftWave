//! Unified error type for `swiftwave_core`.

use thiserror::Error;

/// The canonical error type used throughout `swiftwave_core`.
///
/// Every fallible operation in this crate returns `Result<T, SwiftWaveError>`.
#[derive(Debug, Error)]
pub enum SwiftWaveError {
    // -----------------------------------------------------------------------
    // I/O
    // -----------------------------------------------------------------------
    /// An underlying I/O error (file read, write, seek, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // -----------------------------------------------------------------------
    // Serialisation
    // -----------------------------------------------------------------------
    /// JSON serialisation / deserialisation failure.
    #[error("Serialisation error: {0}")]
    Serialisation(#[from] serde_json::Error),

    // -----------------------------------------------------------------------
    // Cryptography
    // -----------------------------------------------------------------------
    /// The Noise Protocol handshake failed.
    ///
    /// # Security note
    /// This error is intentionally opaque to callers — we never reveal
    /// *why* a handshake failed to prevent oracle attacks.
    #[error("Handshake failed")]
    HandshakeFailed,

    /// Noise Protocol state-machine error (snow).
    #[error("Noise error: {0}")]
    Noise(#[from] snow::Error),

    /// AEAD decryption failure — ciphertext is corrupt or has been tampered.
    ///
    /// # Security note
    /// Do NOT expose chunk indices, offsets, or key material in this error.
    #[error("Decryption failed — data corrupt or tampered")]
    DecryptionFailed,

    /// AEAD encryption failure.
    #[error("Encryption failed")]
    EncryptionFailed,

    /// Chunk hash mismatch — file integrity violation.
    ///
    /// # Security note
    /// This indicates either disk corruption or a malicious sender.
    /// The transfer MUST be aborted and the partial file deleted.
    #[error("Chunk hash mismatch on chunk {chunk_index} of file {file_id}")]
    HashMismatch {
        /// File identifier.
        file_id: String,
        /// Zero-based chunk index where the mismatch was detected.
        chunk_index: u64,
    },

    /// Keypair loading failed — key material is corrupt or missing.
    #[error("Identity error: {0}")]
    Identity(String),

    // -----------------------------------------------------------------------
    // Transport
    // -----------------------------------------------------------------------
    /// A QUIC connection error.
    #[error("QUIC connection error: {0}")]
    QuicConnection(String),

    /// A QUIC stream error.
    #[error("QUIC stream error: {0}")]
    QuicStream(String),

    /// Transport has not been configured / bound yet.
    #[error("Transport not initialised")]
    TransportNotInitialised,

    // -----------------------------------------------------------------------
    // Transfer
    // -----------------------------------------------------------------------
    /// Requested transfer ID does not exist.
    #[error("Unknown transfer ID: {0}")]
    UnknownTransfer(String),

    /// An invalid state-machine transition was attempted.
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition {
        /// Current state.
        from: String,
        /// Attempted target state.
        to: String,
    },

    /// The transfer was cancelled by the user or peer.
    #[error("Transfer cancelled")]
    Cancelled,

    /// Transfer was interrupted unexpectedly.
    #[error("Transfer interrupted: {0}")]
    Interrupted(String),

    // -----------------------------------------------------------------------
    // Capability / negotiation
    // -----------------------------------------------------------------------
    /// Peers could not agree on a common capability set.
    #[error("Capability negotiation failed: {0}")]
    CapabilityMismatch(String),

    // -----------------------------------------------------------------------
    // Platform
    // -----------------------------------------------------------------------
    /// Platform-specific adapter error.
    #[error("Platform error: {0}")]
    Platform(String),

    // -----------------------------------------------------------------------
    // Generic
    // -----------------------------------------------------------------------
    /// An unexpected internal error — used sparingly.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, SwiftWaveError>;
