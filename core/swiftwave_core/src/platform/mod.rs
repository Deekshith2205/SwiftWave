//! Platform adapter — bridges OS-specific networking APIs to swiftwave_core traits.
//!
//! On Android, Wi-Fi Aware and Wi-Fi Direct are exposed via Kotlin JNI bindings
//! (see `native/android/`). On desktop, mDNS-SD and native sockets are used.
//!
//! This module defines the `PlatformAdapter` trait which the Flutter FFI layer
//! instantiates with the correct concrete implementation at runtime.
//!
//! # Supported Platforms
//!
//! | Platform | Discovery        | Transport | Status       |
//! |----------|-----------------|-----------|--------------|
//! | Android  | Wi-Fi Aware/BLE  | QUIC/UDP  | TODO Phase 2 |
//! | Windows  | mDNS (DNS-SD)    | QUIC/UDP  | TODO Phase 2 |
//! | Linux    | mDNS (Avahi)     | QUIC/UDP  | TODO Phase 2 |
//! | macOS    | Bonjour (mDNS)   | QUIC/UDP  | TODO Phase 2 |

use crate::{
    discovery::Discovery,
    transport::Transport,
    Result,
};
use async_trait::async_trait;

/// Runtime platform capabilities query.
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    /// Wi-Fi Aware is available (Android 8+).
    pub wifi_aware: bool,
    /// Wi-Fi Direct (P2P) is available.
    pub wifi_direct: bool,
    /// Bluetooth Low Energy is available.
    pub ble: bool,
    /// mDNS / DNS-SD is available.
    pub mdns: bool,
}

/// The `PlatformAdapter` trait — one implementation per OS target.
///
/// Selected at startup based on compile-time target and runtime capability
/// checks. The adapter is responsible for returning the correct
/// `Discovery` and `Transport` implementations.
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Query what networking capabilities are available on this device.
    async fn capabilities(&self) -> Result<PlatformCapabilities>;

    /// Return the best available discovery backend for this platform.
    async fn discovery_backend(&self) -> Result<Box<dyn Discovery>>;

    /// Return the best available transport backend for this platform.
    async fn transport_backend(&self) -> Result<Box<dyn Transport>>;

    /// Platform name (e.g. `"android"`, `"windows"`).
    fn platform_name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// Stub adapter
// ---------------------------------------------------------------------------

/// A stub platform adapter for cross-platform testing.
///
/// Returns `StubDiscovery` and reports no real capabilities.
pub struct StubPlatformAdapter;

#[async_trait]
impl PlatformAdapter for StubPlatformAdapter {
    async fn capabilities(&self) -> Result<PlatformCapabilities> {
        Ok(PlatformCapabilities {
            wifi_aware: false,
            wifi_direct: false,
            ble: false,
            mdns: false,
        })
    }

    async fn discovery_backend(&self) -> Result<Box<dyn Discovery>> {
        Ok(Box::new(crate::discovery::stub::StubDiscovery::new()))
    }

    async fn transport_backend(&self) -> Result<Box<dyn Transport>> {
        // TODO (Phase 2): Return a real QuicTransport.
        Err(crate::SwiftWaveError::Platform(
            "StubPlatformAdapter has no real transport backend".into(),
        ))
    }

    fn platform_name(&self) -> &'static str {
        "stub"
    }
}
