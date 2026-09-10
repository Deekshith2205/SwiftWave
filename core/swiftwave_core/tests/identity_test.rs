use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use swiftwave_core::device::identity::DeviceIdentity;
use swiftwave_core::device::storage::{SecureStorage, IDENTITY_SECRET_KEY};
use swiftwave_core::error::{Result, SwiftWaveError};

struct TestStorage {
    cache: Mutex<HashMap<String, Vec<u8>>>,
    load_should_fail: bool,
    save_should_fail: bool,
}

impl TestStorage {
    fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            load_should_fail: false,
            save_should_fail: false,
        }
    }
}

impl SecureStorage for TestStorage {
    fn save_secret(&self, key: &str, secret: &[u8]) -> Result<()> {
        if self.save_should_fail {
            return Err(SwiftWaveError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Save failed")));
        }
        self.cache.lock().unwrap().insert(key.to_string(), secret.to_vec());
        Ok(())
    }

    fn load_secret(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if self.load_should_fail {
            return Err(SwiftWaveError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Load failed")));
        }
        Ok(self.cache.lock().unwrap().get(key).cloned())
    }

    fn delete_secret(&self, key: &str) -> Result<()> {
        self.cache.lock().unwrap().remove(key);
        Ok(())
    }
}

#[test]
fn test_1_first_generation() {
    let storage = Arc::new(TestStorage::new());
    let identity = DeviceIdentity::load_or_generate("Test", storage.clone()).unwrap();
    
    let cache = storage.cache.lock().unwrap();
    let stored = cache.get(IDENTITY_SECRET_KEY).unwrap();
    assert_eq!(stored.len(), 32);
    assert_eq!(identity.secret_key_bytes(), stored.as_slice());
}

#[test]
fn test_2_persistence() {
    let storage = Arc::new(TestStorage::new());
    let identity1 = DeviceIdentity::load_or_generate("Test", storage.clone()).unwrap();
    
    let identity2 = DeviceIdentity::load_or_generate("Test", storage.clone()).unwrap();
    
    assert_eq!(identity1.public_key_bytes(), identity2.public_key_bytes());
}

#[test]
fn test_3_fingerprint_stability() {
    let storage = Arc::new(TestStorage::new());
    let identity1 = DeviceIdentity::load_or_generate("Test", storage.clone()).unwrap();
    let identity2 = DeviceIdentity::load_or_generate("Test", storage.clone()).unwrap();
    
    assert_eq!(identity1.fingerprint(), identity2.fingerprint());
}

#[test]
fn test_4_corrupted_length() {
    let storage = Arc::new(TestStorage::new());
    storage.save_secret(IDENTITY_SECRET_KEY, &vec![0u8; 31]).unwrap();
    
    let result = DeviceIdentity::load_or_generate("Test", storage.clone());
    assert!(matches!(result, Err(SwiftWaveError::Identity(_))));
    
    storage.save_secret(IDENTITY_SECRET_KEY, &vec![0u8; 33]).unwrap();
    let result = DeviceIdentity::load_or_generate("Test", storage.clone());
    assert!(matches!(result, Err(SwiftWaveError::Identity(_))));
}

#[test]
fn test_5_empty_secret() {
    let storage = Arc::new(TestStorage::new());
    storage.save_secret(IDENTITY_SECRET_KEY, &[]).unwrap();
    
    let result = DeviceIdentity::load_or_generate("Test", storage.clone());
    assert!(matches!(result, Err(SwiftWaveError::Identity(_))));
}

#[test]
fn test_6_storage_failure() {
    let mut storage = TestStorage::new();
    storage.load_should_fail = true;
    let storage = Arc::new(storage);
    
    let result = DeviceIdentity::load_or_generate("Test", storage.clone());
    assert!(matches!(result, Err(SwiftWaveError::Io(_))));
}

#[test]
fn test_7_save_failure() {
    let mut storage = TestStorage::new();
    storage.save_should_fail = true;
    let storage = Arc::new(storage);
    
    let result = DeviceIdentity::load_or_generate("Test", storage.clone());
    assert!(matches!(result, Err(SwiftWaveError::Io(_))));
}

#[test]
fn test_8_delete() {
    let storage = Arc::new(TestStorage::new());
    DeviceIdentity::load_or_generate("Test", storage.clone()).unwrap();
    assert!(storage.load_secret(IDENTITY_SECRET_KEY).unwrap().is_some());
    
    storage.delete_secret(IDENTITY_SECRET_KEY).unwrap();
    assert!(storage.load_secret(IDENTITY_SECRET_KEY).unwrap().is_none());
}

#[test]
fn test_9_concurrent_initialization() {
    use swiftwave_core::runtime::{SwiftWaveRuntime, InMemoryMockStorage};
    use std::sync::Barrier;
    use std::thread;
    
    // We test that Runtime initialization is concurrency-safe.
    // We spawn 10 threads trying to initialize runtimes using the SAME storage instance.
    let num_threads = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let shared_storage = Arc::new(InMemoryMockStorage::new());
    
    let mut handles = vec![];
    for _ in 0..num_threads {
        let b = barrier.clone();
        let storage = shared_storage.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            let runtime = SwiftWaveRuntime::new_with_storage(storage).unwrap();
            runtime.initialize().unwrap();
            
            // Extract the fingerprint
            let id = runtime.identity.read().unwrap();
            let fp = id.as_ref().unwrap().fingerprint();
            fp.0.clone()
        }));
    }
    
    let mut fingerprints = vec![];
    for h in handles {
        fingerprints.push(h.join().unwrap());
    }
    
    // All should be exactly the same
    let first = &fingerprints[0];
    for fp in &fingerprints {
        assert_eq!(fp, first);
    }
    
    // Verify storage contains exactly 1 secret.
    // Wait, InMemoryMockStorage hides cache. But we can load the secret.
    let secret = shared_storage.load_secret(IDENTITY_SECRET_KEY).unwrap().unwrap();
    assert_eq!(secret.len(), 32);
}

#[test]
fn test_runtime_lifecycle() {
    use swiftwave_core::runtime::{SwiftWaveRuntime, LifecycleState, InMemoryMockStorage};
    
    let shared_storage = Arc::new(InMemoryMockStorage::new());
    
    // First runtime initialization
    let runtime1 = SwiftWaveRuntime::new_with_storage(shared_storage.clone()).unwrap();
    runtime1.initialize().unwrap();
    assert_eq!(*runtime1.state.read().unwrap(), LifecycleState::Initialized);
    
    // Repeated initialize is rejected
    let res = runtime1.initialize();
    assert!(res.is_err());
    
    // Extract fingerprint
    let fp1 = {
        let id = runtime1.identity.read().unwrap();
        id.as_ref().unwrap().fingerprint().0.clone()
    };
    
    // Shutdown clears state
    runtime1.shutdown().unwrap();
    assert_eq!(*runtime1.state.read().unwrap(), LifecycleState::Shutdown);
    assert!(runtime1.identity.read().unwrap().is_none());
    assert!(runtime1.config.read().unwrap().is_none());
    
    // Second runtime uses SAME persistent test storage and recovers same identity
    let runtime2 = SwiftWaveRuntime::new_with_storage(shared_storage.clone()).unwrap();
    runtime2.initialize().unwrap();
    let fp2 = {
        let id = runtime2.identity.read().unwrap();
        id.as_ref().unwrap().fingerprint().0.clone()
    };
    
    assert_eq!(fp1, fp2);
}

#[test]
fn test_fingerprint_deterministic() {
    // We create a fake device identity from a known array of 32 bytes just to verify fingerprint logic
    use swiftwave_core::device::identity::{DeviceIdentity, PublicKeyFingerprint};
    use std::sync::Arc;
    
    let storage = Arc::new(TestStorage::new());
    
    // We just generate two and check they are different (changing public key changes fingerprint)
    let identity1 = DeviceIdentity::generate("Device 1", storage.clone()).unwrap();
    let identity2 = DeviceIdentity::generate("Device 2", storage.clone()).unwrap();
    
    assert_ne!(identity1.public_key_bytes(), identity2.public_key_bytes());
    assert_ne!(identity1.fingerprint(), identity2.fingerprint());
    
    // Test exact derivation
    let raw_pk = identity1.public_key_bytes();
    let expected_hash = blake3::hash(raw_pk);
    let expected_fp = bs58::encode(expected_hash.as_bytes()).into_string();
    assert_eq!(identity1.fingerprint(), PublicKeyFingerprint(expected_fp));
}
