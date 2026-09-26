//! `extern "C"` API surface for Dart FFI.
//!
//! Every function in this module:
//! 1. Converts C types to Rust types.
//! 2. Calls the corresponding `swiftwave_core` function via `SwiftWaveRuntime`.
//! 3. Converts the result back to C types.
//!
//! Error handling: functions that can fail return a `SwiftWaveStatus` status code.
//! All exported functions are wrapped in `catch_unwind` to ensure Rust panics
//! never cross the C ABI.

#![allow(clippy::missing_safety_doc)]

use std::ffi::CString;
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};

use swiftwave_core::error::SwiftWaveError;
use swiftwave_core::runtime::{LifecycleState, SwiftWaveRuntime};

// ---------------------------------------------------------------------------
// Status codes
// ---------------------------------------------------------------------------

/// FFI status codes returned by all fallible functions.
#[repr(C)]
pub enum SwiftWaveStatus {
    /// Operation completed successfully.
    Success = 0,
    /// An invalid or destroyed handle was passed.
    InvalidHandle = 1,
    /// The runtime has not been initialized.
    NotInitialized = 2,
    /// The runtime is already initialized.
    AlreadyInitialized = 3,
    /// An I/O error occurred.
    IoError = 4,
    /// An internal core error occurred or a panic was caught.
    InternalError = 5,
    /// A required pointer argument was null.
    NullPointer = 6,
    /// A string argument was not valid UTF-8.
    InvalidUtf8 = 7,
    /// The runtime is in a shutdown state.
    Shutdown = 8,
    /// The runtime was already shut down.
    AlreadyShutdown = 9,
}

impl From<SwiftWaveError> for SwiftWaveStatus {
    fn from(err: SwiftWaveError) -> Self {
        match err {
            SwiftWaveError::Io(_) => SwiftWaveStatus::IoError,
            _ => SwiftWaveStatus::InternalError,
        }
    }
}

// ---------------------------------------------------------------------------
// Opaque Handle
// ---------------------------------------------------------------------------

/// Opaque handle mapping to a Boxed `SwiftWaveRuntime`.
pub struct SwiftWaveHandle {
    pub(crate) runtime: Box<SwiftWaveRuntime>,
    pub(crate) discovery_task: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

// ---------------------------------------------------------------------------
// Panic Containment Helpers
// ---------------------------------------------------------------------------

/// Executes a closure that returns a `SwiftWaveStatus`, catching any panics.
/// If a panic occurs, returns `SwiftWaveStatus::InternalError`.
pub(crate) fn catch_panic_status<F>(f: F) -> SwiftWaveStatus
where
    F: FnOnce() -> SwiftWaveStatus,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(status) => status,
        Err(_) => SwiftWaveStatus::InternalError,
    }
}

/// Executes a closure that returns a raw pointer, catching any panics.
/// If a panic occurs, returns a null pointer.
pub(crate) fn catch_panic_ptr<T, F>(f: F) -> *mut T
where
    F: FnOnce() -> *mut T,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn create_storage(
    path: &str,
) -> Option<std::sync::Arc<dyn swiftwave_core::device::storage::SecureStorage>> {
    match swiftwave_storage_windows::WindowsSecureStorage::new(path) {
        Ok(s) => Some(std::sync::Arc::new(s)),
        Err(_) => None,
    }
}

#[cfg(not(windows))]
fn create_storage(
    _path: &str,
) -> Option<std::sync::Arc<dyn swiftwave_core::device::storage::SecureStorage>> {
    // Other platforms not implemented yet. Do NOT fallback to mock.
    None
}

/// Create a new, uninitialized SwiftWave runtime.
///
/// Returns an opaque pointer to the `SwiftWaveHandle`.
/// The caller MUST eventually call `swiftwave_destroy` to free memory.
/// Returns NULL on failure to create the runtime.
#[no_mangle]
pub extern "C" fn swiftwave_create(data_directory: *const c_char) -> *mut SwiftWaveHandle {
    catch_panic_ptr(|| {
        // Set up tracing to stderr in debug builds.
        #[cfg(debug_assertions)]
        let _ = tracing_subscriber::fmt::try_init();

        if data_directory.is_null() {
            return std::ptr::null_mut();
        }

        let path_str = unsafe { std::ffi::CStr::from_ptr(data_directory) }.to_str();

        let storage = match path_str {
            Ok(path) if !path.is_empty() => match create_storage(path) {
                Some(s) => s,
                None => return std::ptr::null_mut(),
            },
            _ => return std::ptr::null_mut(),
        };

        match SwiftWaveRuntime::new_with_storage(storage) {
            Ok(runtime) => {
                let handle = Box::new(SwiftWaveHandle {
                    runtime: Box::new(runtime),
                    discovery_task: std::sync::Mutex::new(None),
                });
                Box::into_raw(handle)
            }
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Initialize the SwiftWave runtime.
///
/// MUST be called exactly once before using other functionality.
#[no_mangle]
pub extern "C" fn swiftwave_init(handle: *mut SwiftWaveHandle) -> SwiftWaveStatus {
    catch_panic_status(|| {
        if handle.is_null() {
            return SwiftWaveStatus::NullPointer;
        }

        let h = unsafe { &*handle };

        let state = match h.runtime.state.read() {
            Ok(guard) => *guard,
            Err(_) => return SwiftWaveStatus::InternalError,
        };

        if state != LifecycleState::Created {
            return SwiftWaveStatus::AlreadyInitialized;
        }

        match h.runtime.initialize() {
            Ok(_) => SwiftWaveStatus::Success,
            Err(e) => e.into(),
        }
    })
}

/// Shut down the SwiftWave runtime and stop ongoing operations.
///
/// The handle is still valid after this call and must be freed using `swiftwave_destroy`.
#[no_mangle]
pub extern "C" fn swiftwave_shutdown(handle: *mut SwiftWaveHandle) -> SwiftWaveStatus {
    catch_panic_status(|| {
        if handle.is_null() {
            return SwiftWaveStatus::NullPointer;
        }

        let h = unsafe { &*handle };

        let state = match h.runtime.state.read() {
            Ok(guard) => *guard,
            Err(_) => return SwiftWaveStatus::InternalError,
        };

        if state == LifecycleState::Shutdown {
            return SwiftWaveStatus::AlreadyShutdown;
        }

        if state == LifecycleState::Created {
            return SwiftWaveStatus::NotInitialized;
        }

        // Must explicitly stop discovery FFI task if it's running.
        // If the mutex is poisoned, we recover the guard to ensure cleanup still runs.
        let mut task_guard = match h.discovery_task.lock() {
            Ok(guard) => guard,
            Err(poison_error) => poison_error.into_inner(),
        };
        if let Some(task_handle) = task_guard.take() {
            let _ = h.runtime.stop_discovery();
            let _ = h.runtime.tokio_rt.block_on(task_handle);
        }

        match h.runtime.shutdown() {
            Ok(_) => SwiftWaveStatus::Success,
            Err(e) => e.into(),
        }
    })
}

/// Destroy the SwiftWave handle and free its memory.
///
/// MUST be called exactly once per handle created by `swiftwave_create`.
/// Automatically calls shutdown if it hasn't been called yet.
#[no_mangle]
pub extern "C" fn swiftwave_destroy(handle: *mut SwiftWaveHandle) {
    let _ = catch_panic_status(AssertUnwindSafe(|| {
        if handle.is_null() {
            return SwiftWaveStatus::Success;
        }

        let h = unsafe { &*handle };

        // Explicitly stop and await the discovery task if it exists
        // to prevent detached task execution after handle destruction.
        // If the mutex is poisoned, we recover the guard to ensure cleanup still runs.
        let mut task_guard = match h.discovery_task.lock() {
            Ok(guard) => guard,
            Err(poison_error) => poison_error.into_inner(),
        };

        if let Some(task_handle) = task_guard.take() {
            let _ = h.runtime.stop_discovery();
            let _ = h.runtime.tokio_rt.block_on(task_handle);
        }

        // Recover the Box and let it drop to free memory.
        let h_box = unsafe { Box::from_raw(handle) };
        let _ = h_box.runtime.shutdown();
        SwiftWaveStatus::Success
    }));
}

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// Get the device ID (fingerprint) of this runtime.
///
/// Returns a heap-allocated UTF-8 C string.
/// The caller MUST free the returned string with `swiftwave_free_string`.
/// Returns NULL on failure or if not initialized.
#[no_mangle]
pub extern "C" fn swiftwave_get_device_id(handle: *const SwiftWaveHandle) -> *mut c_char {
    catch_panic_ptr(|| {
        if handle.is_null() {
            return std::ptr::null_mut();
        }

        let h = unsafe { &*handle };

        let state = match h.runtime.state.read() {
            Ok(guard) => *guard,
            Err(_) => return std::ptr::null_mut(),
        };

        if state == LifecycleState::Created || state == LifecycleState::Shutdown {
            return std::ptr::null_mut();
        }

        let identity_guard = match h.runtime.identity.read() {
            Ok(guard) => guard,
            Err(_) => return std::ptr::null_mut(),
        };

        if let Some(identity) = identity_guard.as_ref() {
            let fingerprint = identity.fingerprint().0;
            match CString::new(fingerprint) {
                Ok(cs) => cs.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    })
}

// ---------------------------------------------------------------------------
// Memory management
// ---------------------------------------------------------------------------

/// Free a C string previously returned by any `swiftwave_*` function.
///
/// Passing NULL is a no-op.
#[no_mangle]
pub unsafe extern "C" fn swiftwave_free_string(ptr: *mut c_char) {
    let _ = catch_panic_status(AssertUnwindSafe(|| {
        if !ptr.is_null() {
            let _ = unsafe { CString::from_raw(ptr) };
        }
        SwiftWaveStatus::Success
    }));
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

/// Return the SwiftWave library version as a static C string.
///
/// The returned pointer is `'static` — do NOT free it.
#[no_mangle]
pub extern "C" fn swiftwave_version() -> *const c_char {
    b"0.1.0\0".as_ptr() as *const c_char
}

// ---------------------------------------------------------------------------
// Discovery Events
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct CDiscoveryEvent {
    pub event_type: u8,
    pub fingerprint: [c_char; 65],
    pub display_name: [c_char; 65],
    pub address: [c_char; 65],
    pub medium: u8,
    pub rssi_has_value: u8,
    pub rssi: i8,
    pub protocol_version: u16,
    pub last_seen: u64,
}

fn copy_str_to_c_array(dest: &mut [c_char; 65], src: &str) {
    let mut len = src.len().min(64);
    while len > 0 && !src.is_char_boundary(len) {
        len -= 1;
    }
    let valid_str = &src[..len];
    let bytes = valid_str.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        dest[i] = b as c_char;
    }
    for i in len..65 {
        dest[i] = 0;
    }
}

use swiftwave_core::discovery::DiscoveryEvent;

#[no_mangle]
pub extern "C" fn swiftwave_start_discovery(
    handle: *mut SwiftWaveHandle,
    callback: Option<extern "C" fn(CDiscoveryEvent)>,
) -> SwiftWaveStatus {
    catch_panic_status(|| {
        if handle.is_null() {
            return SwiftWaveStatus::NullPointer;
        }
        let cb = match callback {
            Some(c) => c,
            None => return SwiftWaveStatus::NullPointer,
        };
        let h = unsafe { &*handle };

        let mut task_guard = match h.discovery_task.lock() {
            Ok(g) => g,
            Err(_) => return SwiftWaveStatus::InternalError,
        };
        if task_guard.is_some() {
            return SwiftWaveStatus::InternalError;
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        match h.runtime.start_discovery(tx) {
            Ok(_) => {
                let task_handle = h.runtime.tokio_rt.spawn(async move {
                    while let Some(event) = rx.recv().await {
                        let c_event = match event {
                            DiscoveryEvent::PeerFound(peer) => {
                                let mut fp = [0; 65];
                                let mut dn = [0; 65];
                                let mut addr = [0; 65];
                                copy_str_to_c_array(&mut fp, &peer.fingerprint.0);
                                copy_str_to_c_array(&mut dn, &peer.display_name);
                                copy_str_to_c_array(&mut addr, &peer.address.to_string());
                                CDiscoveryEvent {
                                    event_type: 0,
                                    fingerprint: fp,
                                    display_name: dn,
                                    address: addr,
                                    medium: peer.medium as u8,
                                    rssi_has_value: if peer.rssi.is_some() { 1 } else { 0 },
                                    rssi: peer.rssi.unwrap_or(0),
                                    protocol_version: peer.protocol_version,
                                    last_seen: peer.last_seen,
                                }
                            }
                            DiscoveryEvent::PeerLost(fp_str) => {
                                let mut fp = [0; 65];
                                copy_str_to_c_array(&mut fp, &fp_str.0);
                                CDiscoveryEvent {
                                    event_type: 1,
                                    fingerprint: fp,
                                    display_name: [0; 65],
                                    address: [0; 65],
                                    medium: 0,
                                    rssi_has_value: 0,
                                    rssi: 0,
                                    protocol_version: 0,
                                    last_seen: 0,
                                }
                            }
                        };
                        cb(c_event);
                    }
                });

                *task_guard = Some(task_handle);
                SwiftWaveStatus::Success
            }
            Err(e) => e.into(),
        }
    })
}

#[no_mangle]
pub extern "C" fn swiftwave_stop_discovery(handle: *mut SwiftWaveHandle) -> SwiftWaveStatus {
    catch_panic_status(|| {
        if handle.is_null() {
            return SwiftWaveStatus::NullPointer;
        }
        let h = unsafe { &*handle };

        let mut task_guard = match h.discovery_task.lock() {
            Ok(g) => g,
            Err(_) => return SwiftWaveStatus::InternalError,
        };

        if let Some(task_handle) = task_guard.take() {
            if let Err(e) = h.runtime.stop_discovery() {
                return e.into();
            }
            // Block until the callback consumer fully terminates
            let _ = h.runtime.tokio_rt.block_on(task_handle);
        }

        SwiftWaveStatus::Success
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c_array_to_string(arr: &[c_char; 65]) -> String {
        let mut bytes = Vec::new();
        for &c in arr.iter() {
            if c == 0 {
                break;
            }
            bytes.push(c as u8);
        }
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn test_copy_str_to_c_array_ascii_exact_capacity() {
        let mut dest = [0; 65];
        let src = "A".repeat(64);
        copy_str_to_c_array(&mut dest, &src);
        assert_eq!(c_array_to_string(&dest), src);
        assert_eq!(dest[64], 0);
    }

    #[test]
    fn test_copy_str_to_c_array_ascii_over_capacity() {
        let mut dest = [0; 65];
        let src = "A".repeat(70);
        copy_str_to_c_array(&mut dest, &src);
        assert_eq!(c_array_to_string(&dest), "A".repeat(64));
        assert_eq!(dest[64], 0);
    }

    #[test]
    fn test_copy_str_to_c_array_2byte_utf8_at_boundary() {
        let mut dest = [0; 65];
        let src = format!("{}ñ", "A".repeat(63));
        copy_str_to_c_array(&mut dest, &src);
        assert_eq!(c_array_to_string(&dest), "A".repeat(63));
    }

    #[test]
    fn test_copy_str_to_c_array_3byte_utf8_at_boundary() {
        let mut dest = [0; 65];
        let src = format!("{}€", "A".repeat(63));
        copy_str_to_c_array(&mut dest, &src);
        assert_eq!(c_array_to_string(&dest), "A".repeat(63));
    }

    #[test]
    fn test_copy_str_to_c_array_4byte_utf8_at_boundary() {
        let mut dest = [0; 65];
        let src = format!("{}🌍", "A".repeat(62));
        copy_str_to_c_array(&mut dest, &src);
        assert_eq!(c_array_to_string(&dest), "A".repeat(62));
    }

    #[test]
    fn test_copy_str_to_c_array_multilingual_display_name() {
        let mut dest = [0; 65];
        let src = "こんにちは世界-你好世界-안녕하세요세계";
        copy_str_to_c_array(&mut dest, &src);
        assert_eq!(c_array_to_string(&dest), src);
    }

    #[test]
    fn test_copy_str_to_c_array_emoji_display_name() {
        let mut dest = [0; 65];
        let src = "📱🔥 MacBook Pro (Deekshith) 🚀✨";
        copy_str_to_c_array(&mut dest, &src);
        assert_eq!(c_array_to_string(&dest), src);
    }

    #[test]
    fn test_copy_str_to_c_array_empty_string() {
        let mut dest = [1; 65];
        copy_str_to_c_array(&mut dest, "");
        assert_eq!(c_array_to_string(&dest), "");
        assert_eq!(dest[0], 0);
        assert_eq!(dest[64], 0);
    }
}
