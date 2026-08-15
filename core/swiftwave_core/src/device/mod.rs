//! Device module: identity and capabilities.

pub mod capabilities;
pub mod identity;

pub use identity::{DeviceId, DeviceIdentity};
pub use capabilities::{Capabilities, NegotiatedCapabilities, Transport, CompressionCodec, negotiate};
