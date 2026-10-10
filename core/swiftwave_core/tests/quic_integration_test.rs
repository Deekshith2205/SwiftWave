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

    // Run A in the main thread (using the generic pure transport connect)
    let auth_conn_a = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect(addr_b).await.expect("A failed to connect") });
    let a_peer_identity = auth_conn_a.peer();

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
fn test_quic_handshake_fingerprint_mismatch() {
    let rt_local = SwiftWaveRuntime::new().unwrap();
    let rt_attacker = SwiftWaveRuntime::new().unwrap();
    let rt_victim = SwiftWaveRuntime::new().unwrap();

    rt_local.initialize().unwrap();
    rt_attacker.initialize().unwrap();
    rt_victim.initialize().unwrap();

    let port_attacker = rt_attacker.actual_quic_port().unwrap();
    let addr_attacker: SocketAddr = format!("127.0.0.1:{}", port_attacker).parse().unwrap();

    let victim_fp = rt_victim
        .identity
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .fingerprint();
    let attacker_fp = rt_attacker
        .identity
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .fingerprint();

    // Attacker claims to be the victim in mDNS
    let expected_claim = victim_fp.clone();

    let mut attacker_incoming = rt_attacker.subscribe_incoming_peers();

    // Local connects to attacker, but expects victim's fingerprint
    let result = rt_local.tokio_rt.block_on(async {
        rt_local
            .connect_with_claim(addr_attacker, &expected_claim)
            .await
    });

    // It MUST fail.
    assert!(result.is_err());
    let err = result.err().unwrap();

    // It should specifically be a FingerprintMismatch.
    match err {
        swiftwave_core::error::SwiftWaveError::FingerprintMismatch { expected, actual } => {
            assert_eq!(expected, expected_claim.0);
            assert_eq!(actual, attacker_fp.0);
        }
        _ => panic!("Expected FingerprintMismatch, got {:?}", err),
    }

    // Note: The attacker's runtime will actually successfully authenticate the local node
    // because Noise XX completes fully before the local node verifies the binding and drops
    // the connection. This is mathematically correct for Noise XX. The critical security
    // property is that the local node immediately rejects the connection.
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
    let err = result.err().unwrap();
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
    let b_fp = b_identity.fingerprint();

    // Connect first time
    let mut b_incoming = rt_b.subscribe_incoming_peers();
    let rt_b_clone = rt_b.clone();
    let b_thread = std::thread::spawn(move || {
        rt_b_clone
            .tokio_rt
            .block_on(async move { timeout(Duration::from_secs(5), b_incoming.recv()).await })
    });

    let auth_conn_1 = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect_with_claim(addr_b, &b_fp).await.unwrap() });
    let a_peer_id = auth_conn_1.peer();
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

    let auth_conn_2 = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect_with_claim(addr_b, &b_fp).await.unwrap() });
    let a_peer_id_2 = auth_conn_2.peer();
    assert_eq!(a_peer_id_2.fingerprint, b_identity.fingerprint());
    b_thread_2.join().unwrap().unwrap().unwrap();
}

#[test]
fn test_explicit_local_close_and_remote_disconnect() {
    let rt_a = Arc::new(SwiftWaveRuntime::new().unwrap());
    let rt_b = Arc::new(SwiftWaveRuntime::new().unwrap());

    rt_a.initialize().unwrap();
    rt_b.initialize().unwrap();

    let port_b = rt_b.actual_quic_port().unwrap();
    let addr_b: SocketAddr = format!("127.0.0.1:{}", port_b).parse().unwrap();

    let b_identity = rt_b.identity.read().unwrap().as_ref().unwrap().clone();
    let a_identity = rt_a.identity.read().unwrap().as_ref().unwrap().clone();
    let a_fp = a_identity.fingerprint();
    let b_fp = b_identity.fingerprint();

    // Connect
    let auth_conn_a = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect_with_claim(addr_b, &b_fp).await.unwrap() });

    let b_fp_str = b_fp.0.clone();
    let a_fp_str = a_fp.0.clone();

    // Give the accept task a moment to register inbound connection
    rt_b.tokio_rt.block_on(async {
        wait_for_registry_has(rt_b.clone(), &a_fp_str).await;
    });

    assert!(rt_a.get_connection(&b_fp_str).is_some());
    assert!(rt_b.get_connection(&a_fp_str).is_some());

    // Local close on A
    rt_a.close_connection(&b_fp_str);

    // Wait for monitors to clean up deterministically
    rt_a.tokio_rt.block_on(async {
        wait_for_registry_empty(rt_a.clone(), &b_fp_str).await;
    });
    rt_b.tokio_rt.block_on(async {
        wait_for_registry_empty(rt_b.clone(), &a_fp_str).await;
    });

    // Both should be cleared (A cleared via explicit close, B cleared via remote disconnect detection)
    assert!(rt_a.get_connection(&b_fp_str).is_none());
    assert!(rt_b.get_connection(&a_fp_str).is_none());
}

#[test]
fn test_duplicate_connection_policy() {
    let rt_a = Arc::new(SwiftWaveRuntime::new().unwrap());
    let rt_b = Arc::new(SwiftWaveRuntime::new().unwrap());

    rt_a.initialize().unwrap();
    rt_b.initialize().unwrap();

    let port_b = rt_b.actual_quic_port().unwrap();
    let addr_b: SocketAddr = format!("127.0.0.1:{}", port_b).parse().unwrap();

    let b_fp = rt_b
        .identity
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .fingerprint();

    // Connect 1
    let auth_conn_1 = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect_with_claim(addr_b, &b_fp).await.unwrap() });

    let id_1 = auth_conn_1.id();
    let b_fp_str = b_fp.0.clone();
    rt_a.tokio_rt.block_on(async {
        wait_for_registry_contains(rt_a.clone(), &b_fp_str, id_1).await;
    });

    // Connect 2
    let auth_conn_2 = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect_with_claim(addr_b, &b_fp).await.unwrap() });

    let id_2 = auth_conn_2.id();
    rt_a.tokio_rt.block_on(async {
        wait_for_registry_contains(rt_a.clone(), &b_fp_str, id_2).await;
    });

    assert_ne!(id_1, id_2);

    let current = rt_a.get_connection(&b_fp_str).unwrap();
    assert_eq!(current.id(), id_2);

    // To test the stale monitor race, we need to explicitly wait until A's monitor has fired.
    // However, A's monitor running will NOT change the state since it gets rejected.
    // The most deterministic way is to wait a bounded duration for the closed signal of conn_1,
    // which proves it was closed and its monitor was woken.
    rt_a.tokio_rt.block_on(async {
        let _ = auth_conn_1.quic_connection().closed().await;
        // Yield a few times to let the monitor task run
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;
    });

    // B should still be registered
    let current_after = rt_a.get_connection(&b_fp_str).unwrap();
    assert_eq!(current_after.id(), id_2);
}

#[test]
fn test_runtime_shutdown_with_active_connection() {
    let rt_a = Arc::new(SwiftWaveRuntime::new().unwrap());
    let rt_b = Arc::new(SwiftWaveRuntime::new().unwrap());

    rt_a.initialize().unwrap();
    rt_b.initialize().unwrap();

    let port_b = rt_b.actual_quic_port().unwrap();
    let addr_b: SocketAddr = format!("127.0.0.1:{}", port_b).parse().unwrap();
    let b_fp = rt_b
        .identity
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .fingerprint();
    let a_fp = rt_a
        .identity
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .fingerprint();

    let _auth_conn_a = rt_a
        .tokio_rt
        .block_on(async { rt_a.connect_with_claim(addr_b, &b_fp).await.unwrap() });

    let b_fp_str = b_fp.0.clone();
    let a_fp_str = a_fp.0.clone();

    // Wait for inbound registration
    rt_b.tokio_rt.block_on(async {
        wait_for_registry_has(rt_b.clone(), &a_fp_str).await;
    });

    assert!(rt_a.get_connection(&b_fp_str).is_some());
    assert!(rt_b.get_connection(&a_fp_str).is_some());

    // Shutdown A
    rt_a.shutdown().unwrap();

    // A should clear its registry
    assert!(rt_a.get_connection(&b_fp_str).is_none());

    // Wait for B to detect remote disconnect deterministically
    rt_b.tokio_rt.block_on(async {
        wait_for_registry_empty(rt_b.clone(), &a_fp_str).await;
    });

    // B should clear its registry
    assert!(rt_b.get_connection(&a_fp_str).is_none());
}

async fn wait_for_registry_empty(rt: Arc<SwiftWaveRuntime>, fp: &str) {
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if rt.get_connection(fp).is_none() {
                break;
            }
            rt.wait_for_connection_state_change().await;
        }
    })
    .await;
    assert!(result.is_ok(), "Timeout waiting for registry to empty");
}

async fn wait_for_registry_contains(rt: Arc<SwiftWaveRuntime>, fp: &str, expected_id: u64) {
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(conn) = rt.get_connection(fp) {
                if conn.id() == expected_id {
                    break;
                }
            }
            rt.wait_for_connection_state_change().await;
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "Timeout waiting for registry to contain specific connection"
    );
}

async fn wait_for_registry_has(rt: Arc<SwiftWaveRuntime>, fp: &str) {
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if rt.get_connection(fp).is_some() {
                break;
            }
            rt.wait_for_connection_state_change().await;
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "Timeout waiting for registry to contain connection"
    );
}
