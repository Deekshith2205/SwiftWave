//! File metadata: describes a single file to be transferred.

use serde::{Deserialize, Serialize};

use crate::security::hashing::Hash;
use crate::transfer::session::FileId;

/// Metadata describing a single file in a transfer.
///
/// This is serialised and sent to the peer before any chunks are transferred
/// so the receiver can pre-allocate space and validate the final hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Unique identifier for this file within the transfer.
    pub id: FileId,
    /// Original filename (no path components — sanitised before use).
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// BLAKE3 hash of the entire plaintext file (hex-encoded, 64 chars).
    ///
    /// # Security note
    /// The receiver MUST verify this after all chunks are assembled and
    /// decrypted. A mismatch indicates corruption or tampering.
    pub hash_hex: String,
    /// MIME type (optional, informational only).
    pub mime_type: Option<String>,
    /// Last-modified timestamp of the original file (Unix seconds).
    pub modified_at: Option<u64>,
}

impl FileMetadata {
    /// Construct metadata for a file.
    pub fn new(
        id: FileId,
        name: impl Into<String>,
        size: u64,
        hash: Hash,
        mime_type: Option<String>,
        modified_at: Option<u64>,
    ) -> Self {
        Self {
            id,
            name: sanitise_filename(name.into()),
            size,
            hash_hex: crate::security::hashing::hash_to_hex(&hash),
            mime_type,
            modified_at,
        }
    }

    /// Number of chunks required to transfer this file at `chunk_size` bytes per chunk.
    pub fn chunk_count(&self, chunk_size: usize) -> u64 {
        if self.size == 0 {
            return 0;
        }
        (self.size + chunk_size as u64 - 1) / chunk_size as u64
    }

    /// Decode the stored hash back to raw bytes.
    pub fn hash(&self) -> Option<Hash> {
        crate::security::hashing::hex_to_hash(&self.hash_hex)
    }
}

/// Remove path separators and null bytes from a filename.
///
/// # Security note
/// Receivers MUST sanitise filenames before writing to disk to prevent
/// path-traversal attacks (e.g., `../../etc/passwd`).
/// This function is the minimal defence; platform adapters should apply
/// additional OS-level restrictions.
pub fn sanitise_filename(name: String) -> String {
    name.chars()
        .filter(|&c| c != '/' && c != '\\' && c != '\0')
        .collect::<String>()
        .trim_start_matches('.')
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::hashing::hash_bytes;
    use crate::transfer::session::new_file_id;

    #[test]
    fn chunk_count_exact_multiple() {
        let meta = FileMetadata::new(
            new_file_id(), "test.txt", 1024, hash_bytes(b""), None, None,
        );
        assert_eq!(meta.chunk_count(256), 4);
    }

    #[test]
    fn chunk_count_with_remainder() {
        let meta = FileMetadata::new(
            new_file_id(), "test.txt", 1000, hash_bytes(b""), None, None,
        );
        assert_eq!(meta.chunk_count(256), 4); // ceil(1000/256)
    }

    #[test]
    fn chunk_count_zero_size() {
        let meta = FileMetadata::new(
            new_file_id(), "empty.txt", 0, hash_bytes(b""), None, None,
        );
        assert_eq!(meta.chunk_count(256), 0);
    }

    #[test]
    fn filename_is_sanitised() {
        let meta = FileMetadata::new(
            new_file_id(),
            "../../etc/passwd",
            0,
            hash_bytes(b""),
            None,
            None,
        );
        assert!(!meta.name.contains('/'));
        assert!(!meta.name.starts_with('.'));
    }

    #[test]
    fn metadata_json_roundtrip() {
        let id = new_file_id();
        let hash = hash_bytes(b"file content");
        let meta = FileMetadata::new(id, "photo.jpg", 2048, hash, Some("image/jpeg".into()), None);
        let json = serde_json::to_string(&meta).unwrap();
        let back: FileMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.size, 2048);
        assert_eq!(back.hash_hex, meta.hash_hex);
        assert_eq!(back.mime_type, Some("image/jpeg".into()));
    }
}
