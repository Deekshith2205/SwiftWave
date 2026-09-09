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

use swiftwave_core::runtime::{SwiftWaveRuntime, LifecycleState};
use swiftwave_core::error::SwiftWaveError;

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
    runtime: Box<SwiftWaveRuntime>,
}

// ---------------------------------------------------------------------------
// Panic Containment Helpers
// ---------------------------------------------------------------------------

/// Executes a closure that returns a `SwiftWaveStatus`, catching any panics.
/// If a panic occurs, returns `SwiftWaveStatus::InternalError`.
pub(crate) fn catch_panic_status<F>(f: F) -> SwiftWaveStatus
where
    F: FnOnce() -> SwiftWaveStatus + std::panic::UnwindSafe,
{
    match catch_unwind(f) {
        Ok(status) => status,
        Err(_) => SwiftWaveStatus::InternalError,
    }
}

/// Executes a closure that returns a raw pointer, catching any panics.
/// If a panic occurs, returns a null pointer.
pub(crate) fn catch_panic_ptr<T, F>(f: F) -> *mut T
where
    F: FnOnce() -> *mut T + std::panic::UnwindSafe,
{
    match catch_unwind(f) {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

/// Create a new, uninitialized SwiftWave runtime.
///
/// Returns an opaque pointer to the `SwiftWaveHandle`. 
/// The caller MUST eventually call `swiftwave_destroy` to free memory.
/// Returns NULL on failure to create the runtime.
#[no_mangle]
pub extern "C" fn swiftwave_create() -> *mut SwiftWaveHandle {
    catch_panic_ptr(|| {
        // Set up tracing to stderr in debug builds.
        #[cfg(debug_assertions)]
        let _ = tracing_subscriber::fmt::try_init();

        match SwiftWaveRuntime::new() {
            Ok(runtime) => {
                let handle = Box::new(SwiftWaveHandle {
                    runtime: Box::new(runtime),
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

        // Recover the Box and let it drop to free memory.
        let h = unsafe { Box::from_raw(handle) };
        let _ = h.runtime.shutdown();
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
    catch_panic_ptr(|| {
        b"0.1.0\0".as_ptr() as *const c_char
    })
}
