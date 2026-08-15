//! Transfer session identifiers and session-level metadata.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::device::DeviceId;
use crate::storage::metadata::FileMetadata;

/// Unique identifier for a transfer session (UUID v4).
pub type TransferId = String;

/// Unique identifier for a single file within a transfer.
pub type FileId = String;

/// Allocate a new random transfer ID.
pub fn new_transfer_id() -> TransferId {
    Uuid::new_v4().to_string()
}

/// Allocate a new random file ID.
pub fn new_file_id() -> FileId {
    Uuid::new_v4().to_string()
}

/// Direction of a transfer from the local device's perspective.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferDirection {
    /// We are sending files.
    Outbound,
    /// We are receiving files.
    Inbound,
}

/// A single transfer session, which may contain multiple files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferSession {
    /// Globally unique session ID.
    pub id: TransferId,
    /// Transfer direction.
    pub direction: TransferDirection,
    /// Fingerprint of the remote peer's static public key.
    pub peer_id: DeviceId,
    /// Network address of the remote peer.
    pub peer_address: SocketAddr,
    /// Files included in this transfer.
    pub files: Vec<FileMetadata>,
    /// Total bytes to transfer (sum of all file sizes).
    pub total_bytes: u64,
    /// Unix timestamp when this session was created.
    pub created_at: u64,
    /// Chunk size agreed during capability negotiation.
    pub chunk_size: usize,
}

impl TransferSession {
    /// Create a new outbound transfer session.
    pub fn new_outbound(
        peer_id: DeviceId,
        peer_address: SocketAddr,
        files: Vec<FileMetadata>,
        chunk_size: usize,
    ) -> Self {
        let total_bytes = files.iter().map(|f| f.size).sum();
        Self {
            id: new_transfer_id(),
            direction: TransferDirection::Outbound,
            peer_id,
            peer_address,
            files,
            total_bytes,
            created_at: unix_now(),
            chunk_size,
        }
    }

    /// Create a new inbound transfer session.
    pub fn new_inbound(
        peer_id: DeviceId,
        peer_address: SocketAddr,
        files: Vec<FileMetadata>,
        chunk_size: usize,
    ) -> Self {
        let total_bytes = files.iter().map(|f| f.size).sum();
        Self {
            id: new_transfer_id(),
            direction: TransferDirection::Inbound,
            peer_id,
            peer_address,
            files,
            total_bytes,
            created_at: unix_now(),
            chunk_size,
        }
    }

    /// Total number of chunks across all files.
    pub fn total_chunks(&self) -> u64 {
        self.files
            .iter()
            .map(|f| f.chunk_count(self.chunk_size))
            .sum()
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
