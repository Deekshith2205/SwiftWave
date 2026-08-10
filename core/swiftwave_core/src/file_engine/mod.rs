//! File engine — chunking, hashing, transfer state machine, and reassembly.
//!
//! Files are split into fixed-size chunks (default 256 KiB). Each chunk is
//! independently hashed with **BLAKE3** so that:
//! - Partial transfers are resumable.
//! - Individual chunks can be requested out-of-order (future: parallel streams).
//! - File integrity is verifiable without re-reading the whole file.
//!
//! # TODO (Phase 2)
//! - Implement `Blake3Hasher` using the `blake3` crate.
//! - Implement chunk splitting with `tokio::fs` and zero-copy `sendfile`.
//! - Build the transfer state machine (Pending → Active → Paused → Done | Failed).
//! - Add optional Zstandard compression per chunk.

use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a file transfer session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransferId(Uuid);

impl TransferId {
    /// Generate a new random transfer ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TransferId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TransferId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata describing a file offered for transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOffer {
    /// Unique transfer session ID.
    pub transfer_id: TransferId,
    /// Original filename (no path components).
    pub file_name: String,
    /// Total file size in bytes.
    pub file_size: u64,
    /// BLAKE3 hash of the entire file (hex string).
    ///
    /// # TODO (Phase 2): compute with `blake3::Hasher`.
    pub file_hash: String,
    /// Chunk size in bytes used for this transfer.
    pub chunk_size: u32,
    /// Total number of chunks.
    pub chunk_count: u32,
    /// Whether Zstandard compression is applied.
    pub compressed: bool,
}

impl FileOffer {
    /// Default chunk size: 256 KiB — balances latency and throughput.
    pub const DEFAULT_CHUNK_SIZE: u32 = 256 * 1024;
}

/// State of a transfer from the perspective of either sender or receiver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferState {
    /// Offer sent / received; waiting for acceptance.
    Pending,
    /// Transfer is actively streaming data.
    Active {
        /// Number of bytes transferred so far.
        bytes_done: u64,
    },
    /// Transfer was paused (user action or network interruption).
    Paused {
        /// Number of bytes transferred before pause.
        bytes_done: u64,
    },
    /// Transfer completed successfully.
    Completed,
    /// Transfer failed with an error message.
    Failed(String),
    /// Transfer was cancelled by local or remote user.
    Cancelled,
}

/// The `FileEngine` trait drives the send and receive sides of a file transfer.
#[async_trait]
pub trait FileEngine: Send + Sync {
    /// Prepare a `FileOffer` from a local file path.
    ///
    /// Computes chunk count and optionally a BLAKE3 hash.
    ///
    /// # TODO (Phase 2): Implement with `blake3` + streaming read.
    async fn prepare_offer(&self, path: &std::path::Path) -> Result<FileOffer>;

    /// Send chunks over `stream` according to `offer`.
    ///
    /// Reports progress via `on_progress(bytes_sent)`.
    ///
    /// # TODO (Phase 2): zero-copy chunk pipeline.
    async fn send_file(
        &self,
        offer: &FileOffer,
        path: &std::path::Path,
        stream: &mut dyn crate::transport::Stream,
        on_progress: &dyn Fn(u64) + Send + Sync,
    ) -> Result<()>;

    /// Receive chunks from `stream` and write to `dest_dir`.
    ///
    /// Verifies each chunk's hash before writing.
    ///
    /// # TODO (Phase 2): resumable receive with bitmap of received chunks.
    async fn receive_file(
        &self,
        offer: &FileOffer,
        dest_dir: &std::path::Path,
        stream: &mut dyn crate::transport::Stream,
        on_progress: &dyn Fn(u64) + Send + Sync,
    ) -> Result<std::path::PathBuf>;
}
