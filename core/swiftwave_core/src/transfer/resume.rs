//! Resume bitmap: tracks which chunks have been successfully received.
//!
//! The bitmap is a compact `Vec<u64>` where each bit represents one chunk.
//! Bit `i` is set if chunk `i` has been fully received and verified.
//!
//! # Resume strategy
//! When a transfer is interrupted:
//! 1. The receiver persists the bitmap to disk as JSON.
//! 2. On reconnect, the receiver sends the bitmap to the sender.
//! 3. The sender only re-sends chunks whose bit is `0`.
//!
//! This allows byte-granular resume without re-sending already-verified data.

use serde::{Deserialize, Serialize};

/// A compact bitset tracking which chunks are complete.
///
/// Bit `i` set → chunk `i` received and verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeBitmap {
    /// Number of chunks in the file.
    pub total_chunks: u64,
    /// Packed u64 words; word[i] covers chunks [i*64, (i+1)*64).
    words: Vec<u64>,
}

impl ResumeBitmap {
    /// Create a new all-zero bitmap for `total_chunks` chunks.
    pub fn new(total_chunks: u64) -> Self {
        let words = vec![0u64; Self::words_for(total_chunks)];
        Self {
            total_chunks,
            words,
        }
    }

    /// Mark chunk `index` as complete.
    ///
    /// # Panics
    /// Panics in debug builds if `index >= total_chunks`.
    pub fn mark_complete(&mut self, index: u64) {
        debug_assert!(index < self.total_chunks, "chunk index out of bounds");
        let (word, bit) = Self::position(index);
        if word < self.words.len() {
            self.words[word] |= 1u64 << bit;
        }
    }

    /// Returns `true` if chunk `index` is complete.
    pub fn is_complete(&self, index: u64) -> bool {
        let (word, bit) = Self::position(index);
        word < self.words.len() && (self.words[word] >> bit) & 1 == 1
    }

    /// Returns `true` if all chunks are complete.
    pub fn is_all_complete(&self) -> bool {
        if self.total_chunks == 0 {
            return true;
        }
        let full_words = (self.total_chunks / 64) as usize;
        let remainder = (self.total_chunks % 64) as u32;

        // All full words must be all-ones.
        for &w in &self.words[..full_words] {
            if w != u64::MAX {
                return false;
            }
        }
        // The last partial word must have exactly `remainder` bits set.
        if remainder > 0 {
            let mask = (1u64 << remainder) - 1;
            if self.words[full_words] & mask != mask {
                return false;
            }
        }
        true
    }

    /// Number of chunks still needed (bits that are 0).
    pub fn pending_count(&self) -> u64 {
        self.total_chunks - self.complete_count()
    }

    /// Number of completed chunks.
    pub fn complete_count(&self) -> u64 {
        self.words.iter().map(|w| w.count_ones() as u64).sum()
    }

    /// Return an iterator over chunk indices that are NOT yet complete.
    pub fn pending_indices(&self) -> impl Iterator<Item = u64> + '_ {
        (0..self.total_chunks).filter(|&i| !self.is_complete(i))
    }

    /// Serialise to JSON for persistence.
    pub fn to_json(&self) -> crate::error::Result<String> {
        serde_json::to_string(self).map_err(crate::error::SwiftWaveError::Serialisation)
    }

    /// Deserialise from JSON.
    pub fn from_json(json: &str) -> crate::error::Result<Self> {
        serde_json::from_str(json).map_err(crate::error::SwiftWaveError::Serialisation)
    }

    fn words_for(n: u64) -> usize {
        ((n + 63) / 64) as usize
    }

    fn position(index: u64) -> (usize, u32) {
        ((index / 64) as usize, (index % 64) as u32)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bitmap_has_no_complete_chunks() {
        let bm = ResumeBitmap::new(100);
        assert_eq!(bm.complete_count(), 0);
        assert_eq!(bm.pending_count(), 100);
        assert!(!bm.is_all_complete());
    }

    #[test]
    fn mark_and_check_individual_chunks() {
        let mut bm = ResumeBitmap::new(200);
        bm.mark_complete(0);
        bm.mark_complete(63);
        bm.mark_complete(64);
        bm.mark_complete(199);

        assert!(bm.is_complete(0));
        assert!(bm.is_complete(63));
        assert!(bm.is_complete(64));
        assert!(bm.is_complete(199));
        assert!(!bm.is_complete(1));
        assert!(!bm.is_complete(100));
        assert_eq!(bm.complete_count(), 4);
    }

    #[test]
    fn is_all_complete_when_all_marked() {
        let n = 130u64;
        let mut bm = ResumeBitmap::new(n);
        for i in 0..n {
            bm.mark_complete(i);
        }
        assert!(bm.is_all_complete());
    }

    #[test]
    fn is_all_complete_fails_if_one_missing() {
        let n = 64u64;
        let mut bm = ResumeBitmap::new(n);
        for i in 0..n {
            bm.mark_complete(i);
        }
        // Corrupt one bit.
        bm.words[0] &= !(1u64 << 5);
        assert!(!bm.is_all_complete());
    }

    #[test]
    fn pending_indices_correct() {
        let mut bm = ResumeBitmap::new(5);
        bm.mark_complete(1);
        bm.mark_complete(3);
        let pending: Vec<u64> = bm.pending_indices().collect();
        assert_eq!(pending, vec![0, 2, 4]);
    }

    #[test]
    fn json_roundtrip() {
        let mut bm = ResumeBitmap::new(300);
        for i in (0..300).step_by(7) {
            bm.mark_complete(i);
        }
        let json = bm.to_json().unwrap();
        let back = ResumeBitmap::from_json(&json).unwrap();
        assert_eq!(bm.complete_count(), back.complete_count());
        assert_eq!(bm.words, back.words);
    }

    #[test]
    fn zero_chunk_bitmap_is_complete() {
        let bm = ResumeBitmap::new(0);
        assert!(bm.is_all_complete());
    }
}
