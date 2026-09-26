use std::sync::Arc;
use swiftwave_core::runtime::{InMemoryMockStorage, LifecycleState, SwiftWaveRuntime};

#[test]
fn test_runtime_quic_endpoint_lifecycle() {
    let storage = Arc::new(InMemoryMockStorage::new());
    let runtime = SwiftWaveRuntime::new_with_storage(storage).unwrap();

    // A. Runtime initialization creates the QUIC server endpoint.
    runtime.initialize().unwrap();

    let quic_server_guard = runtime.quic_server.read().unwrap();
    let endpoint = quic_server_guard.as_ref().unwrap();

    // B. The endpoint bound to 0.0.0.0:0 obtains a non-zero actual local port.
    let local_addr = endpoint.local_addr().unwrap();
    let actual_port = local_addr.port();
    assert_ne!(actual_port, 0, "Actual port should be non-zero");

    // Ensure the cached port equals the endpoint port
    let cached_port = runtime.actual_quic_port.read().unwrap().unwrap();
    assert_eq!(
        cached_port, actual_port,
        "Cached port must match endpoint port"
    );

    // Drop the guard before testing discovery
    drop(quic_server_guard);

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

    let quic_server_guard = runtime.quic_server.read().unwrap();
    assert!(
        quic_server_guard.is_none(),
        "QUIC endpoint should be dropped on shutdown"
    );
    drop(quic_server_guard);

    let cached_port_guard = runtime.actual_quic_port.read().unwrap();
    assert!(
        cached_port_guard.is_none(),
        "Cached port should be cleared on shutdown"
    );
    drop(cached_port_guard);

    // F. Repeated shutdown remains safe.
    runtime.shutdown().unwrap(); // Should return Ok(()) without error
}
