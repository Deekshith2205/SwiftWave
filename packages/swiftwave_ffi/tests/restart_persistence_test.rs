use std::ffi::CString;
use tempfile::tempdir;

// Since swiftwave_ffi generates a cdylib, we must be careful linking it in tests,
// but we can import the api module directly if we define it as a lib.
// But swiftwave_create is #[no_mangle] extern "C".

// Instead of trying to link to it dynamically in a rust test, we can just call the public 
// functions exported in api.rs if we make them visible, or just do an unsafe call.
// The easiest is just doing a test inside `packages/swiftwave_ffi/src/api.rs`.
// We will just put the test here and see if `cargo test` picks it up.

#[cfg(windows)]
#[test]
fn test_restart_persistence() {
    // 1. Create directory
    let dir = tempdir().unwrap();
    let path_str = dir.path().to_str().unwrap();
    let c_path = CString::new(path_str).unwrap();

    // First initialization
    let handle1 = swiftwave_ffi::swiftwave_create(c_path.as_ptr());
    assert!(!handle1.is_null());
    
    let status1 = swiftwave_ffi::swiftwave_init(handle1);
    assert_eq!(status1 as i32, 0);

    let id_ptr1 = swiftwave_ffi::swiftwave_get_device_id(handle1);
    assert!(!id_ptr1.is_null());
    let id1 = unsafe { std::ffi::CStr::from_ptr(id_ptr1) }.to_string_lossy().into_owned();
    
    unsafe { swiftwave_ffi::swiftwave_free_string(id_ptr1) };
    swiftwave_ffi::swiftwave_destroy(handle1);

    // Second initialization (Restart) using the exact same path
    let handle2 = swiftwave_ffi::swiftwave_create(c_path.as_ptr());
    assert!(!handle2.is_null());

    let status2 = swiftwave_ffi::swiftwave_init(handle2);
    assert_eq!(status2 as i32, 0);

    let id_ptr2 = swiftwave_ffi::swiftwave_get_device_id(handle2);
    assert!(!id_ptr2.is_null());
    let id2 = unsafe { std::ffi::CStr::from_ptr(id_ptr2) }.to_string_lossy().into_owned();
    
    unsafe { swiftwave_ffi::swiftwave_free_string(id_ptr2) };
    swiftwave_ffi::swiftwave_destroy(handle2);

    // The core assertion: Fingerprints must match.
    assert_eq!(id1, id2);
}
