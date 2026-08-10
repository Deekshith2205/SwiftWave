//! Integration tests for swiftwave_core.
//!
//! These tests exercise the public trait interfaces using stub implementations.
//! No real networking, filesystem access, or cryptography is performed.

use swiftwave_core::{
    discovery::{stub::StubDiscovery, Discovery, DiscoveryEvent},
    identity::{DeviceId, DeviceInfo, StubIdentity, DeviceIdentity},
    platform::{PlatformAdapter, StubPlatformAdapter},
    security::{SecuritySession, StubSession},
};

// ---------------------------------------------------------------------------
// Identity tests
// ---------------------------------------------------------------------------

#[test]
fn device_id_is_unique() {
    let a = DeviceId::generate();
    let b = DeviceId::generate();
    assert_ne!(a, b, "each DeviceId must be unique");
}

#[test]
fn stub_identity_public_key_is_zeroed() {
    let identity = StubIdentity::new("Test Device");
    assert_eq!(identity.public_key_bytes(), [0u8; 32]);
}

#[tokio::test]
async fn stub_identity_sign_returns_error() {
    let identity = StubIdentity::new("Test Device");
    let result = identity.sign(b"hello").await;
    assert!(result.is_err(), "StubIdentity.sign() must return Err");
}

// ---------------------------------------------------------------------------
// Discovery tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stub_discovery_emits_peer_found() {
    let discovery = StubDiscovery::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);

    let local_info = DeviceInfo {
        id: DeviceId::generate(),
        display_name: "Local Test Device".into(),
        app_version: "0.1.0".into(),
    };

    discovery.start(&local_info, tx).await.expect("start should succeed");

    // The stub emits after 100 ms; give it 500 ms to arrive.
    let event = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        rx.recv(),
    )
    .await
    .expect("timeout: no event received")
    .expect("channel closed unexpectedly");

    match event {
        DiscoveryEvent::PeerFound(peer) => {
            assert_eq!(peer.address, "127.0.0.1:7777");
        }
        other => panic!("unexpected event: {:?}", other),
    }

    discovery.stop().await.expect("stop should succeed");
}

// ---------------------------------------------------------------------------
// Security tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stub_session_round_trip() {
    let session = StubSession::new();
    let plaintext = b"swiftwave test payload";

    let ciphertext = session.encrypt(plaintext).await.expect("encrypt failed");
    let decrypted = session.decrypt(&ciphertext).await.expect("decrypt failed");

    assert_eq!(decrypted, plaintext, "stub session must round-trip data");
}

// ---------------------------------------------------------------------------
// Platform adapter tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stub_adapter_capabilities_all_false() {
    let adapter = StubPlatformAdapter;
    let caps = adapter.capabilities().await.expect("capabilities failed");

    assert!(!caps.wifi_aware);
    assert!(!caps.wifi_direct);
    assert!(!caps.ble);
    assert!(!caps.mdns);
}

#[tokio::test]
async fn stub_adapter_discovery_backend_is_stub() {
    let adapter = StubPlatformAdapter;
    let backend = adapter
        .discovery_backend()
        .await
        .expect("discovery_backend failed");

    assert_eq!(backend.backend_name(), "stub");
}

#[tokio::test]
async fn stub_adapter_transport_backend_returns_error() {
    let adapter = StubPlatformAdapter;
    let result = adapter.transport_backend().await;
    assert!(result.is_err(), "stub adapter should have no real transport");
}
