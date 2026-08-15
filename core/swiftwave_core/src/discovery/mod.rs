//! Discovery module: peer discovery trait and types.

pub mod types;

pub use types::{DiscoveredPeer, DiscoveryEvent, DiscoveryMedium};

use async_trait::async_trait;
use tokio::sync::mpsc;

/// A platform-specific discovery backend.
///
/// Implementors broadcast this device's presence and listen for peers.
/// Platform adapters (Android, Linux, Windows, macOS) provide concrete
/// implementations; the core only depends on this trait.
#[async_trait]
pub trait Discovery: Send + Sync {
    /// Start advertising this device and scanning for peers.
    ///
    /// Events are sent over `tx`. The caller receives them via the
    /// corresponding `Receiver`.
    async fn start(&mut self, tx: mpsc::Sender<DiscoveryEvent>) -> crate::error::Result<()>;

    /// Stop advertising and scanning, releasing any held resources.
    async fn stop(&mut self) -> crate::error::Result<()>;
}
