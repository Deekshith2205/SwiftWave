use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use swiftwave_core::runtime::SwiftWaveRuntime;

#[test]
fn test_quic_handshake_success() {
    let rt_a = Arc::new(SwiftWaveRuntime::new().unwrap());
    let rt_b = Arc::new(SwiftWaveRuntime::new().unwrap());

    rt_a.initialize().unwrap();
    rt_b.initialize().unwrap();

    let port_b = rt_b.actual_quic_port().unwrap();
    let addr_b: SocketAddr = format!("127.0.0.1:{}", port_b).parse().unwrap();

    let mut b_incoming = rt_b.subscribe_incoming_peers();

    let b_identity = rt_b.identity.read().unwrap().as_ref().unwrap().clone();
    let a_identity = rt_a.identity.read().unwrap().as_ref().unwrap().clone();

    // Use a background thread for B to wait for connection
    let rt_b_clone = rt_b.clone();
    let b_thread = std::thread::spawn(move || {
        rt_b_clone
            .tokio_rt
            .block_on(async move { timeout(Duration::from_secs(5), b_incoming.recv()).await })
    });

    // Run A in the main thread
    let a_peer_identity = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect(addr_b).await.expect("A failed to connect") });

    // A sees B's identity
    assert_eq!(a_peer_identity.fingerprint, b_identity.fingerprint());
    assert_eq!(a_peer_identity.public_key, *b_identity.public_key_bytes());

    // B sees A's identity
    let b_recv_result = b_thread
        .join()
        .unwrap()
        .expect("Timeout on B")
        .expect("Broadcast closed");
    assert_eq!(b_recv_result.fingerprint, a_identity.fingerprint());
    assert_eq!(b_recv_result.public_key, *a_identity.public_key_bytes());
}

#[test]
fn test_connection_failure_to_unavailable_endpoint() {
    let rt_a = SwiftWaveRuntime::new().unwrap();
    rt_a.initialize().unwrap();

    // Port 0 is usually an unavailable endpoint/impossible, or we can use a known closed port like 1 or some random high port
    let addr: SocketAddr = "127.0.0.1:23456".parse().unwrap();

    let result = rt_a
        .tokio_rt
        .block_on(async { tokio::time::timeout(Duration::from_secs(1), rt_a.connect(addr)).await });

    assert!(result.is_err());
    let err = result.unwrap_err();
    let tokio::time::error::Elapsed { .. } = err;
}

#[test]
fn test_runtime_shutdown_while_active() {
    let rt_a = SwiftWaveRuntime::new().unwrap();
    rt_a.initialize().unwrap();

    // Shutdown runtime cleanly
    rt_a.shutdown().unwrap();

    // Make sure the accept task was aborted
    // Make sure the accept task was aborted
    assert!(
        !rt_a.is_accept_task_active(),
        "Accept task should be taken and aborted"
    );
}

#[test]
fn test_multiple_sequential_connections() {
    let rt_a = Arc::new(SwiftWaveRuntime::new().unwrap());
    let rt_b = Arc::new(SwiftWaveRuntime::new().unwrap());

    rt_a.initialize().unwrap();
    rt_b.initialize().unwrap();

    let port_b = rt_b.actual_quic_port().unwrap();
    let addr_b: SocketAddr = format!("127.0.0.1:{}", port_b).parse().unwrap();

    let b_identity = rt_b.identity.read().unwrap().as_ref().unwrap().clone();

    // Connect first time
    let mut b_incoming = rt_b.subscribe_incoming_peers();
    let rt_b_clone = rt_b.clone();
    let b_thread = std::thread::spawn(move || {
        rt_b_clone
            .tokio_rt
            .block_on(async move { timeout(Duration::from_secs(5), b_incoming.recv()).await })
    });

    let a_peer_id = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect(addr_b).await.unwrap() });
    assert_eq!(a_peer_id.fingerprint, b_identity.fingerprint());
    b_thread.join().unwrap().unwrap().unwrap();

    // Connect second time using same endpoints!
    let mut b_incoming_2 = rt_b.subscribe_incoming_peers();
    let rt_b_clone_2 = rt_b.clone();
    let b_thread_2 = std::thread::spawn(move || {
        rt_b_clone_2
            .tokio_rt
            .block_on(async move { timeout(Duration::from_secs(5), b_incoming_2.recv()).await })
    });

    let a_peer_id_2 = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect(addr_b).await.unwrap() });
    assert_eq!(a_peer_id_2.fingerprint, b_identity.fingerprint());
    b_thread_2.join().unwrap().unwrap().unwrap();
}
