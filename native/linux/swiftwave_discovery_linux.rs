// native/linux/swiftwave_discovery_linux.rs
//
// Linux platform adapter — Avahi mDNS + BlueZ BLE peer discovery.
//
// TODO (Phase 2): Implement using Avahi D-Bus crate or `mdns-sd`.
//
// Compiled only on Linux: #[cfg(target_os = "linux")]

/// Linux mDNS/Avahi discovery adapter.
///
/// # TODO (Phase 2): full implementation
pub struct LinuxDiscovery;
