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
