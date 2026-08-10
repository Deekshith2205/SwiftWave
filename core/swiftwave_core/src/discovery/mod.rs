//! Peer discovery — finding nearby SwiftWave Share devices on the local network.
//!
//! The `Discovery` trait is the single integration point. Concrete adapters
//! (mDNS, Wi-Fi Aware, BLE GATT) implement this trait and are selected by
//! the platform adapter layer.
//!
//! # TODO (Phase 2)
//! - mDNS via `mdns-sd` or `astro-dnssd`
//! - Wi-Fi Aware via Android native adapter (see `native/android/`)
//! - BLE GATT advertisement via `btleplug`

use crate::{identity::DeviceInfo, Result};
use async_trait::async_trait;
use std::time::Duration;

pub mod stub;

/// A peer discovered on the local network.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Public identity of the remote device.
    pub device_info: DeviceInfo,
    /// Transport address at which the peer can be reached.
    /// Format depends on transport: `"192.168.1.5:7777"` for QUIC, etc.
    pub address: String,
    /// Received signal strength indicator, if available (dBm).
    pub rssi: Option<i16>,
}

/// Events emitted by a running discovery session.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// A new peer became visible.
    PeerFound(DiscoveredPeer),
    /// A previously visible peer disappeared.
    PeerLost(String /* device_id as string */),
    /// Discovery engine reported an internal warning (non-fatal).
    Warning(String),
}

/// The `Discovery` trait — implemented by every discovery backend.
///
/// Implementors stream [`DiscoveryEvent`]s via a channel and handle their
/// own lifecycle. Dropping the implementation must stop background tasks.
#[async_trait]
pub trait Discovery: Send + Sync {
    /// Start advertising this device and scanning for peers.
    ///
    /// `tx` is a bounded channel sender; the implementation should not block
    /// if the channel is full — drop events and emit a `Warning` instead.
    async fn start(
        &self,
        local_info: &DeviceInfo,
        tx: tokio::sync::mpsc::Sender<DiscoveryEvent>,
    ) -> Result<()>;

    /// Stop advertising and scanning, releasing all OS resources.
    async fn stop(&self) -> Result<()>;

    /// Return the human-readable name of this discovery backend.
    fn backend_name(&self) -> &'static str;
}
