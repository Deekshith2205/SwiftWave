//! Device module: identity, storage, and capabilities.

pub mod capabilities;
pub mod identity;
pub mod storage;

pub use identity::{DeviceIdentity, PublicKeyFingerprint, PeerIdentity};
pub use capabilities::{Capabilities, NegotiatedCapabilities, Transport, CompressionCodec, negotiate};
pub use storage::{SecureStorage, IDENTITY_SECRET_KEY};
