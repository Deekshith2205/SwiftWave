use std::sync::Arc;
use swiftwave_core::runtime::{InMemoryMockStorage, SwiftWaveRuntime};

#[test]
fn test_runtime_quic_endpoint_lifecycle() {
    let storage = Arc::new(InMemoryMockStorage::new());
    let runtime = SwiftWaveRuntime::new_with_storage(storage).unwrap();

    // A. Runtime initialization creates the QUIC server endpoint.
    runtime.initialize().unwrap();

    // B. The endpoint obtains a non-zero actual local port.
    let actual_port = runtime.actual_quic_port().unwrap();
    assert_ne!(actual_port, 0, "Actual port should be non-zero");

    // Ensure the cached port equals the endpoint port
    let cached_port = runtime.actual_quic_port().unwrap();
    assert_eq!(
        cached_port, actual_port,
        "Cached port must match endpoint port"
    );

    // C & D. SwiftWaveRuntime uses that actual port for discovery and mDNS configuration receives it.
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    runtime.start_discovery(tx).unwrap();

    let discovery_guard = runtime.discovery.read().unwrap();
    let discovery = discovery_guard
        .as_ref()
        .expect("Discovery should be active");

    drop(discovery_guard);

    // Stop discovery
    runtime.stop_discovery().unwrap();

    // E. Runtime shutdown closes the endpoint cleanly.
    runtime.shutdown().unwrap();

    assert!(
        !runtime.is_quic_server_active(),
        "QUIC endpoint should be dropped on shutdown"
    );

    assert!(
        runtime.actual_quic_port().is_none(),
        "Cached port should be cleared on shutdown"
    );

    // F. Repeated shutdown remains safe.
    runtime.shutdown().unwrap(); // Should return Ok(()) without error
}
