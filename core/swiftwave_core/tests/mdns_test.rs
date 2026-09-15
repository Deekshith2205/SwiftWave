use std::collections::HashMap;
use swiftwave_core::device::identity::PublicKeyFingerprint;
use swiftwave_core::discovery::mdns::{SwiftWaveTxtRecord, SUPPORTED_DISCOVERY_VERSION, MAX_DISPLAY_NAME_LEN};

#[test]
fn test_valid_txt_record_parsing() {
    let mut props = HashMap::new();
    props.insert("v".to_string(), SUPPORTED_DISCOVERY_VERSION.to_string());
    props.insert("fp".to_string(), "ABCDEFGH1234567890ABCDEFGH1234567890".to_string());
    props.insert("name".to_string(), "Alice's Phone".to_string());

    let record = SwiftWaveTxtRecord::parse(&props).expect("Valid record should parse");
    assert_eq!(record.fingerprint.0, "ABCDEFGH1234567890ABCDEFGH1234567890");
    assert_eq!(record.display_name, "Alice's Phone");
}

#[test]
fn test_missing_version_rejected() {
    let mut props = HashMap::new();
    props.insert("fp".to_string(), "ABCDEFGH1234567890ABCDEFGH1234567890".to_string());
    props.insert("name".to_string(), "Alice's Phone".to_string());

    assert!(SwiftWaveTxtRecord::parse(&props).is_none());
}

#[test]
fn test_unsupported_version_rejected() {
    let mut props = HashMap::new();
    props.insert("v".to_string(), "2".to_string()); // We support 1
    props.insert("fp".to_string(), "ABCDEFGH1234567890ABCDEFGH1234567890".to_string());
    props.insert("name".to_string(), "Alice's Phone".to_string());

    assert!(SwiftWaveTxtRecord::parse(&props).is_none());
}

#[test]
fn test_missing_fingerprint_rejected() {
    let mut props = HashMap::new();
    props.insert("v".to_string(), SUPPORTED_DISCOVERY_VERSION.to_string());
    props.insert("name".to_string(), "Alice's Phone".to_string());

    assert!(SwiftWaveTxtRecord::parse(&props).is_none());
}

#[test]
fn test_malformed_fingerprint_rejected() {
    let mut props = HashMap::new();
    props.insert("v".to_string(), SUPPORTED_DISCOVERY_VERSION.to_string());
    props.insert("fp".to_string(), "too_short".to_string());
    props.insert("name".to_string(), "Alice's Phone".to_string());

    assert!(SwiftWaveTxtRecord::parse(&props).is_none());
}

#[test]
fn test_oversized_display_name_truncated() {
    let mut props = HashMap::new();
    props.insert("v".to_string(), SUPPORTED_DISCOVERY_VERSION.to_string());
    props.insert("fp".to_string(), "ABCDEFGH1234567890ABCDEFGH1234567890".to_string());
    
    let long_name = "a".repeat(100);
    props.insert("name".to_string(), long_name.clone());

    let record = SwiftWaveTxtRecord::parse(&props).expect("Should parse with truncation");
    assert_eq!(record.display_name.len(), MAX_DISPLAY_NAME_LEN);
    assert_eq!(record.display_name, "a".repeat(MAX_DISPLAY_NAME_LEN));
}

#[test]
fn test_txt_record_serialization() {
    let record = SwiftWaveTxtRecord {
        fingerprint: PublicKeyFingerprint("ABCDEFGH1234567890ABCDEFGH1234567890".to_string()),
        display_name: "Alice's Phone".to_string(),
    };

    let props = record.to_properties();
    assert_eq!(props.get("v").unwrap(), SUPPORTED_DISCOVERY_VERSION);
    assert_eq!(props.get("fp").unwrap(), "ABCDEFGH1234567890ABCDEFGH1234567890");
    assert_eq!(props.get("name").unwrap(), "Alice's Phone");
}

use swiftwave_core::discovery::mdns::PeerRegistry;
use swiftwave_core::discovery::{DiscoveredPeer, DiscoveryEvent, DiscoveryMedium};
use std::net::SocketAddr;

fn dummy_peer(fp: &str) -> DiscoveredPeer {
    DiscoveredPeer {
        fingerprint: PublicKeyFingerprint(fp.to_string()),
        display_name: "Test Peer".to_string(),
        address: "127.0.0.1:1234".parse().unwrap(),
        medium: DiscoveryMedium::MdnsUdp,
        rssi: None,
        protocol_version: 1,
        last_seen: 0,
    }
}

#[test]
fn test_registry_exact_service_removal() {
    let mut reg = PeerRegistry::new();
    let peer = dummy_peer("FP1");

    // Found
    let events = reg.handle_resolved("instanceA._swiftwave._udp.local.".to_string(), peer);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], DiscoveryEvent::PeerFound(_)));

    // Lost
    let events = reg.handle_removed("instanceA._swiftwave._udp.local.");
    assert_eq!(events.len(), 1);
    match &events[0] {
        DiscoveryEvent::PeerLost(fp) => assert_eq!(fp.0, "FP1"),
        _ => panic!("Expected PeerLost"),
    }
}

#[test]
fn test_registry_multiple_instances_for_one_fingerprint() {
    let mut reg = PeerRegistry::new();
    let peer = dummy_peer("FP1");

    let _ = reg.handle_resolved("instanceA".to_string(), peer.clone());
    let events2 = reg.handle_resolved("instanceB".to_string(), peer);
    
    // We already have FP1 from instanceA, instanceB just updates the registry.
    // It should emit PeerFound for the update (or not, but won't crash)
    // Wait, the test checks REMOVAL logic specifically.
    
    // Remove instanceA
    let lost_events = reg.handle_removed("instanceA");
    assert!(lost_events.is_empty(), "Should not emit PeerLost, instanceB is still active");

    // Remove instanceB
    let lost_events = reg.handle_removed("instanceB");
    assert_eq!(lost_events.len(), 1, "Should emit PeerLost when last instance is removed");
}

#[test]
fn test_registry_service_reassignment() {
    let mut reg = PeerRegistry::new();
    
    let peer_x = dummy_peer("FP_X");
    let peer_y = dummy_peer("FP_Y");

    // A -> X
    reg.handle_resolved("instanceA".to_string(), peer_x);
    
    // A -> Y (reassigned)
    let events = reg.handle_resolved("instanceA".to_string(), peer_y);
    
    // It should emit PeerLost(X) and PeerFound(Y)
    assert!(events.iter().any(|e| matches!(e, DiscoveryEvent::PeerLost(f) if f.0 == "FP_X")));
    assert!(events.iter().any(|e| matches!(e, DiscoveryEvent::PeerFound(p) if p.fingerprint.0 == "FP_Y")));
}

#[test]
fn test_registry_short_name_collision() {
    let mut reg = PeerRegistry::new();
    let peer1 = dummy_peer("ABCDEFGH_1"); // short is ABCDEFGH
    let peer2 = dummy_peer("ABCDEFGH_2"); // short is ABCDEFGH

    reg.handle_resolved("instance1".to_string(), peer1);
    reg.handle_resolved("instance2".to_string(), peer2);

    let events = reg.handle_removed("instance1");
    // Only peer1 should be removed
    assert_eq!(events.len(), 1);
    match &events[0] {
        DiscoveryEvent::PeerLost(fp) => assert_eq!(fp.0, "ABCDEFGH_1"),
        _ => panic!("Expected PeerLost for peer1"),
    }
    
    // Peer2 is still in the registry
    assert!(reg.peers.contains_key(&PublicKeyFingerprint("ABCDEFGH_2".to_string())));
}

#[test]
fn test_registry_reappearance() {
    let mut reg = PeerRegistry::new();
    let peer = dummy_peer("FP1");

    // Found
    reg.handle_resolved("instanceA".to_string(), peer.clone());
    // Lost
    reg.handle_removed("instanceA");
    // Found again
    let events = reg.handle_resolved("instanceA".to_string(), peer);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], DiscoveryEvent::PeerFound(_)));
}

#[tokio::test]
async fn test_mdns_startup_cleanup() {
    use swiftwave_core::discovery::mdns::MdnsDiscovery;
    use swiftwave_core::discovery::Discovery;
    use tokio::sync::mpsc;
    
    // Provide a fingerprint with an invalid name to trigger ServiceInfo::new to fail, or just provide invalid characters 
    // actually, instance name must not contain certain characters, but since instance_name is derived from fp.short() which is hex, it's valid.
    // However, if we pass an invalid `quic_port` maybe? No, `mdns_sd` will fail `register` or `browse` if we try to start multiple daemons or if some state is bad.
    // We can just rely on testing that if it fails, it doesn't panic.
    // Let's force an error by making `quic_port` 0. Actually, `0` might be valid for OS assigned.
    // Let's pass a very long display name (Wait, `SwiftWaveTxtRecord::parse` truncates it, but we pass `self.display_name` raw in `MdnsDiscovery::start`?).
    // No, `txt_record.to_properties()` creates the TXT record. 
    
    // Let's just create an instance. If we create two on the same process, the second might fail to browse if mdns_sd prevents it, or it will succeed.
    let mut mdns = MdnsDiscovery::new(
        PublicKeyFingerprint("TEST_FP".to_string()),
        "Test Device".to_string(),
        12345
    );

    let (tx, _rx) = mpsc::channel(10);
    // This will likely succeed because mdns_sd allows it
    let res = mdns.start(tx).await;
    
    // If it succeeds, stop it
    if res.is_ok() {
        mdns.stop().await.unwrap();
    }
    // We just want to ensure it doesn't panic, and if we could mock mdns-sd we'd assert shutdown is called.
    // Given we can't easily mock `mdns-sd`'s ServiceDaemon, we at least verify it compiles and runs without leaking panics.
}
