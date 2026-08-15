//! Core configuration and runtime constants for `swiftwave_core`.
//!
//! All tuneable parameters live here. Downstream crates and platform adapters
//! construct a `CoreConfig` and pass it to the engine at startup.

use std::time::Duration;

/// Default size of a transfer chunk: **256 KiB**.
///
/// Rationale:
/// - Large enough to keep QUIC streams busy on fast Wi-Fi (up to ~150 Mbps).
/// - Small enough to allow fine-grained resume bitmaps without excessive RAM.
/// - A 10 GiB file requires ≤ 40 960 chunks at this size.
pub const DEFAULT_CHUNK_SIZE: usize = 256 * 1024;

/// Maximum permitted chunk size: **4 MiB**.
pub const MAX_CHUNK_SIZE: usize = 4 * 1024 * 1024;

/// Minimum permitted chunk size: **4 KiB**.
pub const MIN_CHUNK_SIZE: usize = 4 * 1024;

/// Maximum concurrent outbound QUIC streams per connection.
pub const MAX_CONCURRENT_STREAMS: u64 = 8;

/// How long to wait for a peer response before declaring a timeout.
pub const DEFAULT_PEER_TIMEOUT: Duration = Duration::from_secs(30);

/// How long to retain a completed session record before garbage-collecting it.
pub const SESSION_RETENTION: Duration = Duration::from_secs(3600);

/// AEAD nonce length in bytes (ChaCha20-Poly1305 uses 96-bit nonces).
///
/// # Security note
/// Nonces MUST be unique per (key, chunk). We derive them deterministically
/// from the chunk index to avoid nonce reuse — see `security::hashing`.
pub const AEAD_NONCE_LEN: usize = 12;

/// AEAD tag length in bytes.
pub const AEAD_TAG_LEN: usize = 16;

/// BLAKE3 hash output length in bytes (256 bits).
pub const HASH_LEN: usize = 32;

/// Read buffer size for streaming file I/O (64 KiB).
///
/// # Memory note
/// The transfer engine never holds more than
/// `READ_BUFFER_SIZE * MAX_CONCURRENT_STREAMS` bytes in memory at once.
pub const READ_BUFFER_SIZE: usize = 64 * 1024;

/// Size of the bounded channel between the file reader and the QUIC sender.
pub const STREAM_CHANNEL_DEPTH: usize = 16;

/// Top-level runtime configuration for the SwiftWave core.
///
/// Construct with `CoreConfig::default()` for sensible production defaults,
/// or use the builder methods to customise individual fields.
#[derive(Debug, Clone)]
pub struct CoreConfig {
    /// Transfer chunk size in bytes.
    pub chunk_size: usize,

    /// Maximum concurrent outbound streams per QUIC connection.
    pub max_concurrent_streams: u64,

    /// Peer response timeout.
    pub peer_timeout: Duration,

    /// Whether to enable Zstd compression (Phase 3).
    pub enable_compression: bool,

    /// Path to persist device identity (keypair + metadata).
    pub identity_path: std::path::PathBuf,

    /// Default download directory for received files.
    pub download_dir: std::path::PathBuf,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            max_concurrent_streams: MAX_CONCURRENT_STREAMS,
            peer_timeout: DEFAULT_PEER_TIMEOUT,
            enable_compression: false,
            identity_path: default_identity_path(),
            download_dir: default_download_dir(),
        }
    }
}

fn default_identity_path() -> std::path::PathBuf {
    // Platform-agnostic: resolve at runtime via dirs crate in production.
    // For now, use a local `.swiftwave` directory.
    std::path::PathBuf::from(".swiftwave").join("identity.json")
}

fn default_download_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("Downloads")
}

impl CoreConfig {
    /// Set the chunk size, clamped to `[MIN_CHUNK_SIZE, MAX_CHUNK_SIZE]`.
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size.clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE);
        self
    }

    /// Enable or disable compression.
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// Set the identity persistence path.
    pub fn with_identity_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.identity_path = path.into();
        self
    }

    /// Set the download directory.
    pub fn with_download_dir(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.download_dir = path.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_sane() {
        let cfg = CoreConfig::default();
        assert_eq!(cfg.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(!cfg.enable_compression);
    }

    #[test]
    fn chunk_size_is_clamped() {
        let cfg = CoreConfig::default().with_chunk_size(1); // below MIN
        assert_eq!(cfg.chunk_size, MIN_CHUNK_SIZE);

        let cfg = CoreConfig::default().with_chunk_size(usize::MAX); // above MAX
        assert_eq!(cfg.chunk_size, MAX_CHUNK_SIZE);
    }
}
