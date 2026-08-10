//! # swiftwave_core
//!
//! The transport-agnostic, platform-independent core of SwiftWave Share.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                      swiftwave_core                          │
//! │                                                          │
//! │  ┌──────────┐  ┌──────────┐  ┌───────────┐             │
//! │  │ identity │  │discovery │  │ security  │             │
//! │  └──────────┘  └──────────┘  └───────────┘             │
//! │        │              │              │                   │
//! │        └──────────────┴──────────────┘                  │
//! │                       │                                  │
//! │               ┌───────────────┐                          │
//! │               │  transport    │                          │
//! │               └───────────────┘                          │
//! │                       │                                  │
//! │               ┌───────────────┐                          │
//! │               │  file_engine  │                          │
//! │               └───────────────┘                          │
//! │                       │                                  │
//! │               ┌───────────────┐                          │
//! │               │   storage     │                          │
//! │               └───────────────┘                          │
//! │                                                          │
//! │  ┌──────────────────────────────────────────────────┐   │
//! │  │              platform (adapters)                  │   │
//! │  └──────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! Each module exposes **traits only**. Concrete implementations live
//! behind those traits, making it straightforward to swap transports,
//! discovery mechanisms, and cipher suites without touching the rest of
//! the codebase.
//!
//! ## No internet, no cloud, no accounts.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

pub mod discovery;
pub mod error;
pub mod file_engine;
pub mod identity;
pub mod platform;
pub mod security;
pub mod storage;
pub mod transport;

// Re-export the most commonly used types at the crate root.
pub use error::{SwiftWaveError, Result};
pub use identity::{DeviceId, DeviceIdentity};
