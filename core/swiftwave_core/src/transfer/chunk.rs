//! Chunk metadata: the unit of transfer in SwiftWave.
//!
//! Files are split into fixed-size chunks. Each chunk carries enough metadata
//! to be encrypted, integrity-checked, and reassembled independently, which
//! enables:
//! - Parallel streaming over multiple QUIC streams.
//! - Granular resume (only re-send unacknowledged chunks).
//! - Per-chunk integrity verification before writing to disk.

use serde::{Deserialize, Serialize};

use crate::security::hashing::{hash_to_hex, hex_to_hash, Hash};
use crate::transfer::session::{FileId, TransferId};

/// Metadata describing a single chunk of a file transfer.
///
/// This struct is serialised and sent ahead of (or alongside) the chunk
/// payload so the receiver can verify integrity and write to the correct
/// offset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// ID of the enclosing transfer session.
    pub transfer_id: TransferId,
    /// ID of the file within that session.
    pub file_id: FileId,
    /// Zero-based index of this chunk within the file.
    pub chunk_index: u64,
    /// Byte offset of this chunk within the file (= chunk_index * chunk_size).
    pub offset: u64,
    /// Number of bytes in this chunk (may be < chunk_size for the last chunk).
    pub length: u32,
    /// BLAKE3 hash of the **plaintext** chunk (hex-encoded, 64 chars).
    ///
    /// # Security note
    /// The receiver verifies this after decryption. A mismatch means
    /// the data was tampered with or corrupted in transit.
    pub hash_hex: String,
    /// Whether this chunk's payload has been Zstd-compressed.
    pub compressed: bool,
    /// Encryption info: nonce used for ChaCha20-Poly1305 (hex-encoded, 24 chars).
    ///
    /// Nonces are derived deterministically; see `security::hashing::derive_chunk_nonce`.
    pub nonce_hex: String,
}

impl ChunkMetadata {
    /// Construct chunk metadata from raw values.
    pub fn new(
        transfer_id: TransferId,
        file_id: FileId,
        chunk_index: u64,
        offset: u64,
        length: u32,
        hash: Hash,
        nonce: &[u8; 12],
        compressed: bool,
    ) -> Self {
        Self {
            transfer_id,
            file_id,
            chunk_index,
            offset,
            length,
            hash_hex: hash_to_hex(&hash),
            compressed,
            nonce_hex: nonce.iter().map(|b| format!("{b:02x}")).collect(),
        }
    }

    /// Decode the stored hash back to raw bytes.
    pub fn hash(&self) -> Option<Hash> {
        hex_to_hash(&self.hash_hex)
    }

    /// Decode the stored nonce back to raw bytes.
    pub fn nonce(&self) -> Option<[u8; 12]> {
        let bytes = decode_hex_12(&self.nonce_hex)?;
        Some(bytes)
    }
}

fn decode_hex_12(s: &str) -> Option<[u8; 12]> {
    if s.len() != 24 {
        return None;
    }
    let mut out = [0u8; 12];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Generate the ordered list of `ChunkMetadata` for a file, given its
/// plaintext content (as a byte slice, for testing) and pre-computed hashes.
///
/// In production, hashes are computed incrementally while reading from disk.
/// This function is used in tests and for metadata-only operations.
pub fn generate_chunk_plan(
    transfer_id: &str,
    file_id: &str,
    file_size: u64,
    chunk_size: usize,
    hashes: &[Hash],
) -> Vec<ChunkMetadata> {
    let n = hashes.len();
    (0..n as u64)
        .map(|i| {
            let offset = i * chunk_size as u64;
            let remaining = file_size.saturating_sub(offset);
            let length = (chunk_size as u64).min(remaining) as u32;
            // Derive a deterministic nonce for each chunk.
            let nonce = crate::security::hashing::derive_chunk_nonce(
                transfer_id.as_bytes(),
                file_id.as_bytes(),
                i,
            );
            ChunkMetadata::new(
                transfer_id.to_string(),
                file_id.to_string(),
                i,
                offset,
                length,
                hashes[i as usize],
                &nonce,
                false,
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::hashing::hash_chunk;

    fn dummy_hash() -> Hash {
        hash_chunk(b"test chunk data")
    }

    #[test]
    fn chunk_metadata_roundtrip_json() {
        let nonce = [0xABu8; 12];
        let meta = ChunkMetadata::new(
            "tid-001".into(),
            "fid-001".into(),
            0,
            0,
            1024,
            dummy_hash(),
            &nonce,
            false,
        );
        let json = serde_json::to_string(&meta).unwrap();
        let back: ChunkMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.chunk_index, 0);
        assert_eq!(back.length, 1024);
        assert_eq!(back.hash(), meta.hash());
        assert_eq!(back.nonce(), meta.nonce());
    }

    #[test]
    fn nonce_roundtrip() {
        let nonce = [0x12u8; 12];
        let meta = ChunkMetadata::new(
            "t".into(), "f".into(), 0, 0, 64, dummy_hash(), &nonce, false,
        );
        assert_eq!(meta.nonce().unwrap(), nonce);
    }

    #[test]
    fn chunk_plan_covers_all_bytes() {
        let file_size = 1_000u64;
        let chunk_size = 256;
        let tid = "transfer-xyz";
        let fid = "file-abc";

        // Pre-compute hashes for each chunk.
        let data = vec![0u8; file_size as usize];
        let hashes: Vec<Hash> = data
            .chunks(chunk_size)
            .map(|c| hash_chunk(c))
            .collect();

        let plan = generate_chunk_plan(tid, fid, file_size, chunk_size, &hashes);

        // 4 chunks (256 + 256 + 256 + 232)
        assert_eq!(plan.len(), 4);
        // Last chunk length
        assert_eq!(plan[3].length, 1000 - 3 * 256);
        // Offsets are correct
        for (i, chunk) in plan.iter().enumerate() {
            assert_eq!(chunk.offset, i as u64 * chunk_size as u64);
        }
    }

    #[test]
    fn chunk_plan_single_chunk() {
        let file_size = 100u64;
        let chunk_size = 256;
        let data = vec![1u8; file_size as usize];
        let hashes = vec![hash_chunk(&data)];
        let plan = generate_chunk_plan("t", "f", file_size, chunk_size, &hashes);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].length, 100);
        assert_eq!(plan[0].offset, 0);
    }
}
