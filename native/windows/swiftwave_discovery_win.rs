// native/windows/swiftwave_discovery_win.rs
//
// Windows platform adapter — mDNS + UDP broadcast peer discovery.
//
// TODO (Phase 2): Implement using one of:
//   a) `mdns-sd` Rust crate (pure Rust, cross-platform mDNS responder)
//   b) Windows DNS-SD API via raw WinAPI FFI
//
// This file is compiled only on Windows:
//   #[cfg(target_os = "windows")]
//
// It will implement the `Discovery` trait from swiftwave_core::discovery.

// TODO (Phase 2): implement
// use swiftwave_core::discovery::{Discovery, DiscoveredPeer, DiscoveryEvent};
// use swiftwave_core::identity::DeviceInfo;
// use swiftwave_core::Result;
// use async_trait::async_trait;

/// Windows mDNS discovery adapter.
///
/// Uses the Windows built-in mDNS responder (available since Windows 10 1703)
/// or falls back to UDP broadcast on port 7779.
///
/// # TODO (Phase 2): full implementation
pub struct WindowsDiscovery;
