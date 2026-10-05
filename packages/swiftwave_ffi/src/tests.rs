use std::ffi::CStr;
use std::ptr;

use crate::api::*;

#[test]
fn test_version() {
    let ptr = swiftwave_version();
    assert!(!ptr.is_null());
    let version_str = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
    assert_eq!(version_str, "0.1.0");
}

#[cfg(windows)]
#[test]
fn test_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    // 1. Create
    let handle = swiftwave_create(c_path);
    assert!(!handle.is_null());

    // 2. Double create gives new handle
    let handle2 = swiftwave_create(c_path);
    assert!(!handle2.is_null());
    assert_ne!(handle, handle2);
    swiftwave_destroy(handle2);

    // 7. Get device ID before init returns null
    assert!(swiftwave_get_device_id(handle).is_null());

    // 3. Init
    let status = swiftwave_init(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    // 4. Duplicate init
    let status = swiftwave_init(handle);
    assert!(matches!(status, SwiftWaveStatus::AlreadyInitialized));

    // 8. Get device ID after init
    let device_id_ptr = swiftwave_get_device_id(handle);
    assert!(!device_id_ptr.is_null());
    let device_id_str = unsafe { CStr::from_ptr(device_id_ptr) }.to_str().unwrap();
    assert!(!device_id_str.is_empty());

    // Free string
    unsafe { swiftwave_free_string(device_id_ptr) };

    // 5. Shutdown
    let status = swiftwave_shutdown(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    // 9. Get device ID after shutdown returns null
    assert!(swiftwave_get_device_id(handle).is_null());

    // 6. Duplicate shutdown
    let status = swiftwave_shutdown(handle);
    assert!(matches!(status, SwiftWaveStatus::AlreadyShutdown));

    // 10. Destroy
    swiftwave_destroy(handle);
}

#[cfg(not(windows))]
#[test]
fn test_create_unsupported_platform_returns_null() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    // 1. Create on unsupported platform should return null
    let handle = swiftwave_create(c_path);
    assert!(handle.is_null());
}

#[test]
fn test_null_handles() {
    assert!(matches!(
        swiftwave_init(ptr::null_mut()),
        SwiftWaveStatus::NullPointer
    ));
    assert!(matches!(
        swiftwave_shutdown(ptr::null_mut()),
        SwiftWaveStatus::NullPointer
    ));
    assert!(swiftwave_get_device_id(ptr::null_mut()).is_null());

    // Destroying null pointer shouldn't panic
    swiftwave_destroy(ptr::null_mut());

    // Freeing null pointer shouldn't panic
    unsafe { swiftwave_free_string(ptr::null_mut()) };
}

#[test]
fn test_panic_containment() {
    // Prove that catch_panic_status and catch_panic_ptr map panics correctly
    // Since we cannot easily inject a panic into the actual C-ABI functions without
    // modifying their logic, we will directly test the internal helper macros.

    let status = crate::api::catch_panic_status(|| {
        panic!("Deliberate test panic!");
    });
    assert!(matches!(status, SwiftWaveStatus::InternalError));

    let ptr = crate::api::catch_panic_ptr(|| {
        panic!("Deliberate test panic!");
        #[allow(unreachable_code)]
        std::ptr::null_mut::<std::os::raw::c_char>()
    });
    assert!(ptr.is_null());
}

#[cfg(windows)]
#[test]
fn test_discovery_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    assert!(!handle.is_null());

    let status = swiftwave_init(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    extern "C" fn dummy_callback(_event: CDiscoveryEvent) {}

    // Start discovery
    let status = swiftwave_start_discovery(handle, Some(dummy_callback));
    assert!(matches!(status, SwiftWaveStatus::Success));

    // Double start should fail
    let status = swiftwave_start_discovery(handle, Some(dummy_callback));
    assert!(matches!(status, SwiftWaveStatus::InternalError));

    // Stop discovery
    let status = swiftwave_stop_discovery(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    // Double stop should succeed but do nothing, actually wait, our implementation returns Success if task_guard was None, wait:
    // "if let Some(task_handle) = task_guard.take() { ... } SwiftWaveStatus::Success"
    let status = swiftwave_stop_discovery(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    swiftwave_shutdown(handle);
    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_discovery_destroy_with_task() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    swiftwave_init(handle);

    extern "C" fn dummy_callback(_event: CDiscoveryEvent) {}
    swiftwave_start_discovery(handle, Some(dummy_callback));

    // 1. destroy with discovery task present
    // 5. callback consumer JoinHandle is fully awaited before destruction
    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_discovery_destroy_without_task() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    swiftwave_init(handle);

    // 2. destroy without discovery task
    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_discovery_shutdown_followed_by_destroy() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    swiftwave_init(handle);

    extern "C" fn dummy_callback(_event: CDiscoveryEvent) {}
    swiftwave_start_discovery(handle, Some(dummy_callback));

    // 3. shutdown followed by destroy
    // 4. repeated cleanup safety
    let status = swiftwave_shutdown(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    // duplicate shutdown
    let status2 = swiftwave_shutdown(handle);
    assert!(matches!(status2, SwiftWaveStatus::AlreadyShutdown));

    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_discovery_destroy_poisoned_mutex_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    swiftwave_init(handle);

    extern "C" fn dummy_callback(_event: CDiscoveryEvent) {}
    swiftwave_start_discovery(handle, Some(dummy_callback));

    // Deliberately poison the mutex by panicking while holding the lock
    let h = unsafe { &*handle };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = h.discovery_task.lock().unwrap();
        panic!("deliberate poison");
    }));

    assert!(h.discovery_task.is_poisoned());

    // Destroy must recover the poisoned mutex, await the task, and not leak
    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_authenticated_peers_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    assert!(!handle.is_null());

    let status = swiftwave_init(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    extern "C" fn dummy_callback(_event: CAuthenticatedPeerEvent) {}

    let status = swiftwave_subscribe_authenticated_peers(handle, Some(dummy_callback));
    assert!(matches!(status, SwiftWaveStatus::Success));

    let status = swiftwave_subscribe_authenticated_peers(handle, Some(dummy_callback));
    assert!(matches!(status, SwiftWaveStatus::InternalError));

    let status = swiftwave_stop_authenticated_peers(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    let status = swiftwave_stop_authenticated_peers(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    swiftwave_shutdown(handle);
    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_authenticated_peers_destroy_with_task() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    swiftwave_init(handle);

    extern "C" fn dummy_callback(_event: CAuthenticatedPeerEvent) {}
    swiftwave_subscribe_authenticated_peers(handle, Some(dummy_callback));

    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_authenticated_peers_shutdown_followed_by_destroy() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    swiftwave_init(handle);

    extern "C" fn dummy_callback(_event: CAuthenticatedPeerEvent) {}
    swiftwave_subscribe_authenticated_peers(handle, Some(dummy_callback));

    let status = swiftwave_shutdown(handle);
    assert!(matches!(status, SwiftWaveStatus::Success));

    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_authenticated_peers_destroy_poisoned_mutex_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let mut path = dir.path().to_str().unwrap().to_string();
    path.push('\0');
    let c_path = path.as_ptr() as *const std::os::raw::c_char;

    let handle = swiftwave_create(c_path);
    swiftwave_init(handle);

    extern "C" fn dummy_callback(_event: CAuthenticatedPeerEvent) {}
    swiftwave_subscribe_authenticated_peers(handle, Some(dummy_callback));

    let h = unsafe { &*handle };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = h.auth_task.lock().unwrap();
        panic!("deliberate poison");
    }));

    assert!(h.auth_task.is_poisoned());

    swiftwave_destroy(handle);
}

#[cfg(windows)]
#[test]
fn test_authenticated_peers_real_callback_delivery() {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    // Global state for callback capture
    static CALLBACK_DATA: Mutex<Option<(String, String)>> = Mutex::new(None);
    *CALLBACK_DATA.lock().unwrap() = None;

    extern "C" fn real_auth_callback(event: CAuthenticatedPeerEvent) {
        let fp = unsafe { CStr::from_ptr(event.fingerprint.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let dn = unsafe { CStr::from_ptr(event.display_name.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        *CALLBACK_DATA.lock().unwrap() = Some((fp, dn));
    }

    // 1. Create server SwiftWave runtime/handle
    let dir_s = tempfile::tempdir().unwrap();
    let mut path_s = dir_s.path().to_str().unwrap().to_string();
    path_s.push('\0');
    let server_handle = swiftwave_create(path_s.as_ptr() as *const std::os::raw::c_char);
    swiftwave_init(server_handle);

    // 2. Subscribe authenticated-peer callback on the server
    swiftwave_subscribe_authenticated_peers(server_handle, Some(real_auth_callback));

    // 3. Create independent client runtime/handle
    let dir_c = tempfile::tempdir().unwrap();
    let mut path_c = dir_c.path().to_str().unwrap().to_string();
    path_c.push('\0');
    let client_handle = swiftwave_create(path_c.as_ptr() as *const std::os::raw::c_char);
    swiftwave_init(client_handle);

    // Get client's fingerprint to assert later
    let client_fp_ptr = swiftwave_get_device_id(client_handle);
    let client_fp = unsafe { CStr::from_ptr(client_fp_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { swiftwave_free_string(client_fp_ptr) };

    let server_h = unsafe { &*server_handle };
    let client_h = unsafe { &*client_handle };

    // 4. Connect client to the server's actual QUIC endpoint
    let server_port = server_h.runtime.actual_quic_port().unwrap();
    let server_addr = format!("127.0.0.1:{}", server_port).parse().unwrap();

    // 5. Complete Noise XX authentication
    let _ = client_h
        .runtime
        .tokio_rt
        .block_on(client_h.runtime.connect(server_addr))
        .expect("Failed to connect client to server");

    // 6. Wait for the server's authenticated-peer callback with a bounded timeout
    let start = Instant::now();
    let mut invoked = false;
    let mut received_fp = String::new();
    let mut received_dn = String::new();

    while start.elapsed() < Duration::from_secs(5) {
        if let Some((fp, dn)) = CALLBACK_DATA.lock().unwrap().clone() {
            received_fp = fp;
            received_dn = dn;
            invoked = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // 7. Assert callback was invoked
    assert!(invoked, "Callback was never invoked");

    // 8. Assert the received fingerprint matches the client's authenticated identity
    assert_eq!(received_fp, client_fp, "Fingerprint mismatch");

    // 9. Assert display name is correct/expected
    assert_eq!(received_dn, "Unknown Peer", "Display name mismatch");

    // 10. Stop subscription
    swiftwave_stop_authenticated_peers(server_handle);

    // Verify lifecycle regression: no further callbacks after stop
    *CALLBACK_DATA.lock().unwrap() = None;
    let _ = client_h
        .runtime
        .tokio_rt
        .block_on(client_h.runtime.connect(server_addr))
        .expect("Failed to connect client to server");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        CALLBACK_DATA.lock().unwrap().is_none(),
        "Callback was invoked after stop returned"
    );

    // 11. Cleanly destroy both handles
    swiftwave_shutdown(client_handle);
    swiftwave_destroy(client_handle);
    swiftwave_shutdown(server_handle);
    swiftwave_destroy(server_handle);
}
