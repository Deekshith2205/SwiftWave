//! # swiftwave_core
//!
//! The transport-agnostic, platform-independent core of SwiftWave.
//!
//! ## Architecture
//!
//! ```text
//! â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
//! â”‚                  swiftwave_core                  â”‚
//! â”‚                                                  â”‚
//! â”‚  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”‚
//! â”‚  â”‚  device  â”‚  â”‚ discovery â”‚  â”‚   security   â”‚  â”‚
//! â”‚  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â”‚
//! â”‚        â”‚              â”‚               â”‚           â”‚
//! â”‚        â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜           â”‚
//! â”‚                       â”‚                           â”‚
//! â”‚               â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”                   â”‚
//! â”‚               â”‚   transport   â”‚                   â”‚
//! â”‚               â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜                   â”‚
//! â”‚                       â”‚                           â”‚
//! â”‚               â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”                   â”‚
//! â”‚               â”‚   transfer    â”‚                   â”‚
//! â”‚               â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜                   â”‚
//! â”‚                       â”‚                           â”‚
//! â”‚  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”‚
//! â”‚  â”‚ storage  â”‚  â”‚  compression  â”‚  â”‚  config   â”‚  â”‚
//! â”‚  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â”‚
//! â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
//! ```
//!
//! Each module exposes **traits only** at its boundary. Concrete
//! implementations live behind those traits, making it straightforward to
//! swap transports, discovery mechanisms, and cipher suites.
//!
//! ## Security model
//! - Every device generates a persistent X25519 keypair on first run.
//! - Peers authenticate via a **Noise_XX** handshake; no CA required.
//! - All file data is encrypted with ChaCha20-Poly1305.
//! - File integrity is verified with BLAKE3 per-chunk and for the whole file.
//! - No internet, no cloud, no user accounts.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

pub mod compression;
pub mod config;
pub mod device;
pub mod discovery;
pub mod error;
pub mod security;
pub mod storage;
pub mod transfer;
pub mod transport;

// Re-export most commonly used types.
pub use config::CoreConfig;
pub use device::identity::DeviceIdentity;
pub mod runtime;
pub use error::{Result, SwiftWaveError};
pub use transfer::session::TransferId;
