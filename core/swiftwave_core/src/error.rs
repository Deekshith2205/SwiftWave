//! Unified error type for swiftwave_core.

use thiserror::Error;

/// The canonical error type used throughout `swiftwave_core`.
#[derive(Debug, Error)]
pub enum SwiftWaveError {
    // --- Identity ---
    /// Failed to generate or load a device key pair.
    #[error("identity error: {0}")]
    Identity(String),

    // --- Discovery ---
    /// Discovery subsystem encountered an error.
    #[error("discovery error: {0}")]
    Discovery(String),

    // --- Security / Crypto ---
    /// Noise handshake or AEAD operation failed.
    #[error("security error: {0}")]
    Security(String),

    // --- Transport ---
    /// Low-level transport (QUIC / TCP) error.
    #[error("transport error: {0}")]
    Transport(String),

    // --- File Engine ---
    /// File hashing, chunking, or reassembly error.
    #[error("file engine error: {0}")]
    FileEngine(String),

    // --- Storage ---
    /// Filesystem I/O error.
    #[error("storage error: {0}")]
    Storage(#[from] std::io::Error),

    // --- Platform ---
    /// Platform-specific adapter error.
    #[error("platform error: {0}")]
    Platform(String),

    // --- Generic ---
    /// An unexpected internal error (should not reach production).
    #[error("internal error: {0}")]
    Internal(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, SwiftWaveError>;
