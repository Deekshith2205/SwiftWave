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

#[test]
fn test_lifecycle() {
    // 1. Create
    let handle = swiftwave_create();
    assert!(!handle.is_null());

    // 2. Double create gives new handle
    let handle2 = swiftwave_create();
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

#[test]
fn test_null_handles() {
    assert!(matches!(swiftwave_init(ptr::null_mut()), SwiftWaveStatus::NullPointer));
    assert!(matches!(swiftwave_shutdown(ptr::null_mut()), SwiftWaveStatus::NullPointer));
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
