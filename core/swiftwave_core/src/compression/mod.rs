//! Compression abstraction (Phase 3 placeholder).
//!
//! This module defines the `Compress` / `Decompress` traits that the transfer
//! engine uses. In Phase 1 and 2, only the `NoCompression` passthrough is
//! active. Phase 3 will add a `ZstdCodec` implementation.

use crate::error::Result;

/// A compression codec that can compress and decompress byte slices.
pub trait Codec: Send + Sync {
    /// Compress `input` and return the compressed bytes.
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Decompress `input` and return the original bytes.
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>>;

    /// Human-readable codec name (e.g. `"none"`, `"zstd-3"`).
    fn name(&self) -> &'static str;

    /// Whether this codec actually transforms the data (false for `NoCompression`).
    fn is_active(&self) -> bool;
}

/// A passthrough codec that copies data unchanged.
///
/// Used when compression is disabled or when the peer does not support Zstd.
pub struct NoCompression;

impl Codec for NoCompression {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        Ok(input.to_vec())
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        Ok(input.to_vec())
    }

    fn name(&self) -> &'static str {
        "none"
    }

    fn is_active(&self) -> bool {
        false
    }
}

/// Select the appropriate codec based on whether compression is enabled.
pub fn select_codec(enabled: bool) -> Box<dyn Codec> {
    if enabled {
        // TODO (Phase 3): return ZstdCodec once implemented.
        tracing::warn!("Compression requested but Zstd is not yet implemented; falling back to none");
        Box::new(NoCompression)
    } else {
        Box::new(NoCompression)
    }
}
