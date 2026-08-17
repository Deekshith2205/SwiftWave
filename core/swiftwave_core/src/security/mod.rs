//! Security module: Noise handshake, BLAKE3 hashing, SAS, and peer trust.

pub mod handshake;
pub mod hashing;
pub mod identity;
pub mod sas;
pub mod session;

pub use hashing::{
    Hash, StreamingHasher, derive_chunk_nonce, hash_bytes, hash_chunk,
    hash_to_hex, hex_to_hash, verify_chunk,
};
pub use identity::{PeerRecord, TrustLevel, TrustStore};
pub use sas::SASGenerator;
pub use session::SecureSession;
pub use handshake::HandshakeState;
