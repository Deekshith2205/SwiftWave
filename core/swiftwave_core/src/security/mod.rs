//! Security module: Noise handshake, BLAKE3 hashing, SAS, and peer trust.

pub mod handshake;
pub mod hashing;
pub mod identity;
pub mod sas;

pub use hashing::{
    Hash, StreamingHasher, derive_chunk_nonce, hash_bytes, hash_chunk,
    hash_to_hex, hex_to_hash, verify_chunk,
};
pub use identity::{PeerFingerprint, PeerRecord, TrustLevel, TrustStore};
pub use sas::{derive_sas_hash, sas_emoji, sas_numeric};
