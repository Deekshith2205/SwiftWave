// native/macos/swiftwave_discovery_macos.rs
//
// macOS platform adapter — Bonjour / DNS-SD peer discovery.
//
// TODO (Phase 2): Implement using the `dnssd` or `mdns-sd` crate.
//
// Compiled only on macOS: #[cfg(target_os = "macos")]

/// macOS Bonjour discovery adapter.
///
/// # TODO (Phase 2): full implementation
pub struct MacosDiscovery;
