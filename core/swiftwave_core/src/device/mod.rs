//! Device module: identity, storage, and capabilities.

pub mod capabilities;
pub mod identity;
pub mod storage;

pub use capabilities::{
    negotiate, Capabilities, CompressionCodec, NegotiatedCapabilities, Transport,
};
pub use identity::{DeviceIdentity, PeerIdentity, PublicKeyFingerprint};
pub use storage::{SecureStorage, IDENTITY_SECRET_KEY};
