//! Discovery types: network-agnostic peer advertisement and metadata.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::device::DeviceId;

/// The network medium over which a peer was discovered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMedium {
    /// UDP multicast / mDNS on local Wi-Fi.
    MdnsUdp,
    /// Android Wi-Fi Aware (NAN).
    WifiAware,
    /// Wi-Fi Direct (P2P group).
    WifiDirect,
    /// Bluetooth Low Energy advertisement.
    Ble,
    /// Manual IP entry.
    Manual,
}

/// A peer discovered on the local network or via a proximity protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    /// Stable fingerprint of the peer's static public key.
    pub device_id: DeviceId,
    /// Human-readable device name advertised by the peer.
    pub display_name: String,
    /// Network address to connect to (may be a link-local IPv6 address).
    pub address: SocketAddr,
    /// How the peer was found.
    pub medium: DiscoveryMedium,
    /// RSSI signal strength in dBm (None if unavailable).
    pub rssi: Option<i8>,
    /// Peer's SwiftWave protocol version.
    pub protocol_version: u16,
    /// Timestamp this record was last refreshed (Unix seconds).
    pub last_seen: u64,
}

impl DiscoveredPeer {
    /// Returns `true` if the record is stale (not refreshed in > 30 s).
    pub fn is_stale(&self, now_unix: u64) -> bool {
        now_unix.saturating_sub(self.last_seen) > 30
    }
}

/// Event emitted by a discovery backend.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// A new peer appeared or an existing record was updated.
    PeerFound(DiscoveredPeer),
    /// A peer's advertisement expired or it disconnected.
    PeerLost(DeviceId),
}
