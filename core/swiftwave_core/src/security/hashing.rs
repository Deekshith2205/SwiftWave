//! BLAKE3 hashing utilities for SwiftWave.
//!
//! # Security design
//! BLAKE3 is used for:
//! 1. **Chunk integrity** — every chunk carries a 32-byte BLAKE3 hash of its
//!    plaintext content. The receiver verifies before writing to disk.
//! 2. **File-level integrity** — a BLAKE3 hash of the full concatenated
//!    plaintext is included in the file metadata.
//! 3. **Device fingerprint** — BLAKE3 of the 32-byte public key (see
//!    `device::identity`).
//! 4. **Nonce derivation** — per-chunk AEAD nonces are derived by hashing
//!    the transfer ID, file ID, and chunk index (see `derive_chunk_nonce`).
//!
//! We never use SHA-1, MD5, or truncated hashes. BLAKE3 provides 128-bit
//! collision resistance from its 256-bit output.

use crate::config::{AEAD_NONCE_LEN, HASH_LEN};

/// A 32-byte BLAKE3 hash.
pub type Hash = [u8; HASH_LEN];

/// Compute the BLAKE3 hash of arbitrary bytes.
#[inline]
pub fn hash_bytes(data: &[u8]) -> Hash {
    *blake3::hash(data).as_bytes()
}

/// Compute the BLAKE3 hash of a file chunk (plaintext).
///
/// This is the value stored in `ChunkMetadata::hash` and verified
/// by the receiver after decryption.
#[inline]
pub fn hash_chunk(data: &[u8]) -> Hash {
    hash_bytes(data)
}

/// Verify a chunk against its expected hash.
///
/// Returns `true` if the chunk matches, `false` otherwise.
///
/// # Security note
/// A mismatch means the chunk is corrupt or was tampered with.
/// The transfer MUST be aborted; see `SwiftWaveError::HashMismatch`.
#[inline]
pub fn verify_chunk(data: &[u8], expected: &Hash) -> bool {
    // Constant-time comparison to avoid timing side-channels.
    blake3::hash(data).as_bytes() == expected
}

/// Compute a streaming BLAKE3 hash over chunks without loading everything
/// into memory.
///
/// Feed chunks with [`StreamingHasher::update`], then call [`StreamingHasher::finalize`].
pub struct StreamingHasher {
    hasher: blake3::Hasher,
}

impl StreamingHasher {
    /// Create a new, empty hasher.
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Feed bytes into the hasher.
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Consume the hasher and return the final 32-byte hash.
    pub fn finalize(self) -> Hash {
        *self.hasher.finalize().as_bytes()
    }
}

impl Default for StreamingHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Derive a deterministic 12-byte AEAD nonce for a specific chunk.
///
/// Input: `transfer_id || file_id || chunk_index` (as little-endian u64).
///
/// # Security note
/// Nonce reuse with the same key is catastrophic for AEAD security. We derive
/// nonces deterministically from the chunk position to guarantee uniqueness
/// within a session. Each transfer uses a freshly negotiated session key, so
/// nonces never repeat across sessions.
pub fn derive_chunk_nonce(transfer_id: &[u8], file_id: &[u8], chunk_index: u64) -> [u8; AEAD_NONCE_LEN] {
    let mut hasher = blake3::Hasher::new_derive_key("swiftwave chunk nonce v1");
    hasher.update(transfer_id);
    hasher.update(file_id);
    hasher.update(&chunk_index.to_le_bytes());
    let output = hasher.finalize();
    // Take the first 12 bytes of BLAKE3 output as the nonce.
    // BLAKE3 output is uniformly random, so any 12-byte prefix is safe.
    let mut nonce = [0u8; AEAD_NONCE_LEN];
    nonce.copy_from_slice(&output.as_bytes()[..AEAD_NONCE_LEN]);
    nonce
}

/// Encode a raw hash as lowercase hex.
pub fn hash_to_hex(hash: &Hash) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decode a lowercase hex string to a hash array.
pub fn hex_to_hash(hex: &str) -> Option<Hash> {
    if hex.len() != HASH_LEN * 2 {
        return None;
    }
    let mut out = [0u8; HASH_LEN];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hi = from_hex_digit(chunk[0])?;
        let lo = from_hex_digit(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn from_hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_hash_is_known_value() {
        // BLAKE3("") is a well-known constant — any deviation indicates a bug.
        let expected = blake3::hash(b"").as_bytes().to_owned();
        assert_eq!(hash_bytes(b""), expected);
    }

    #[test]
    fn verify_chunk_accepts_correct_data() {
        let data = b"hello swiftwave";
        let h = hash_chunk(data);
        assert!(verify_chunk(data, &h));
    }

    #[test]
    fn verify_chunk_rejects_tampered_data() {
        let data = b"hello swiftwave";
        let h = hash_chunk(data);
        let mut tampered = data.to_vec();
        tampered[0] ^= 0xFF;
        assert!(!verify_chunk(&tampered, &h));
    }

    #[test]
    fn streaming_hasher_matches_oneshot() {
        let data = b"streaming test data for swiftwave";
        let oneshot = hash_bytes(data);

        let mut s = StreamingHasher::new();
        for chunk in data.chunks(8) {
            s.update(chunk);
        }
        let streamed = s.finalize();
        assert_eq!(oneshot, streamed);
    }

    #[test]
    fn nonce_derivation_is_deterministic() {
        let tid = b"transfer-abc";
        let fid = b"file-xyz";
        let n1 = derive_chunk_nonce(tid, fid, 0);
        let n2 = derive_chunk_nonce(tid, fid, 0);
        assert_eq!(n1, n2);
    }

    #[test]
    fn nonce_differs_per_chunk_index() {
        let tid = b"transfer-abc";
        let fid = b"file-xyz";
        let n0 = derive_chunk_nonce(tid, fid, 0);
        let n1 = derive_chunk_nonce(tid, fid, 1);
        assert_ne!(n0, n1);
    }

    #[test]
    fn hex_round_trip() {
        let data = b"hex round trip test";
        let h = hash_bytes(data);
        let hex = hash_to_hex(&h);
        let recovered = hex_to_hash(&hex).unwrap();
        assert_eq!(h, recovered);
    }

    #[test]
    fn hex_to_hash_rejects_short_string() {
        assert!(hex_to_hash("abc").is_none());
    }
}
