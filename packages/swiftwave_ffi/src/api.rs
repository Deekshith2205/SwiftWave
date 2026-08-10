//! `extern "C"` API surface for Dart FFI.
//!
//! Every function in this module:
//! 1. Converts C types to Rust types.
//! 2. Calls the corresponding `swiftwave_core` function.
//! 3. Converts the result back to C types.
//!
//! Error handling: functions that can fail return a `SwiftWaveStatus` status code
//! (see below). Detailed error messages are retrievable via
//! `swiftwave_ffi_last_error`.
//!
//! # TODO (Phase 2): replace ad-hoc status codes with a proper handle table.

#![allow(clippy::missing_safety_doc)] // Safety docs are on the trait level above.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Tokio runtime (shared across all FFI calls)
// ---------------------------------------------------------------------------

/// Global Tokio runtime initialised once at `swiftwave_ffi_init`.
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get().expect("swiftwave_ffi_init() was not called before using the API")
}

// ---------------------------------------------------------------------------
// Status codes
// ---------------------------------------------------------------------------

/// FFI status codes returned by all fallible functions.
#[repr(C)]
pub enum SwiftWaveStatus {
    /// Operation completed successfully.
    Ok = 0,
    /// A null pointer was passed where a valid pointer was required.
    NullPointer = 1,
    /// The provided string was not valid UTF-8.
    InvalidUtf8 = 2,
    /// The Rust core returned an error (retrieve via `swiftwave_ffi_last_error`).
    CoreError = 3,
    /// The FFI library has not been initialised (`swiftwave_ffi_init` not called).
    NotInitialised = 4,
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

/// Initialise the SwiftWave FFI library.
///
/// MUST be called once before any other `swiftwave_ffi_*` function.
/// Safe to call from any thread; subsequent calls are no-ops.
///
/// # Returns
/// `SwiftWaveStatus::Ok` on success.
#[no_mangle]
pub extern "C" fn swiftwave_ffi_init() -> SwiftWaveStatus {
    // Set up tracing to stderr in debug builds.
    #[cfg(debug_assertions)]
    let _ = tracing_subscriber::fmt::try_init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)           // keep RAM usage low
        .thread_name("swiftwave-worker")
        .enable_all()
        .build();

    match rt {
        Ok(runtime) => {
            let _ = RUNTIME.set(runtime); // no-op if already set
            SwiftWaveStatus::Ok
        }
        Err(_) => SwiftWaveStatus::CoreError,
    }
}

/// Shut down the SwiftWave FFI library and release all resources.
///
/// After this call, all swiftwave_ffi_* functions are undefined behaviour.
///
/// # TODO (Phase 2): gracefully stop all active transfers first.
#[no_mangle]
pub extern "C" fn swiftwave_ffi_shutdown() {
    // Runtime is dropped when the OnceLock is cleaned up at process exit.
    // TODO (Phase 2): explicit shutdown with transfer cancellation.
}

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// Generate a new device ID and return it as a heap-allocated UTF-8 C string.
///
/// # Ownership
/// The caller MUST free the returned string with `swiftwave_ffi_free_string`.
/// Returns NULL on failure.
///
/// # TODO (Phase 2): persist identity to secure storage.
#[no_mangle]
pub extern "C" fn swiftwave_ffi_generate_device_id() -> *mut c_char {
    let id = swiftwave_core::identity::DeviceId::generate();
    let s = id.to_string();
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------
// Memory management
// ---------------------------------------------------------------------------

/// Free a C string previously returned by any `swiftwave_ffi_*` function.
///
/// Passing NULL is a no-op. Passing a pointer not returned by this library
/// is undefined behaviour.
///
/// # Safety
/// `ptr` must have been returned by this library and not yet freed.
#[no_mangle]
pub unsafe extern "C" fn swiftwave_ffi_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: ptr was allocated by CString::into_raw in this library.
        let _ = CString::from_raw(ptr);
    }
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

/// Return the swiftwave_ffi library version as a static C string.
///
/// The returned pointer is `'static` — do NOT free it.
#[no_mangle]
pub extern "C" fn swiftwave_ffi_version() -> *const c_char {
    // SAFETY: literal is null-terminated and static.
    b"0.1.0\0".as_ptr() as *const c_char
}

// ---------------------------------------------------------------------------
// TODO stubs — Phase 2
// ---------------------------------------------------------------------------

/// Start peer discovery.
///
/// # TODO (Phase 2): wire to `PlatformAdapter::discovery_backend()`.
#[no_mangle]
pub extern "C" fn swiftwave_ffi_start_discovery() -> SwiftWaveStatus {
    SwiftWaveStatus::NotInitialised // TODO (Phase 2)
}

/// Stop peer discovery.
///
/// # TODO (Phase 2)
#[no_mangle]
pub extern "C" fn swiftwave_ffi_stop_discovery() -> SwiftWaveStatus {
    SwiftWaveStatus::NotInitialised // TODO (Phase 2)
}

/// Initiate a file send to the specified peer.
///
/// # TODO (Phase 2): wire to `FileEngine::prepare_offer()` + `Transport::connect()`.
#[no_mangle]
pub extern "C" fn swiftwave_ffi_send_file(
    _peer_id: *const c_char,
    _file_path: *const c_char,
) -> SwiftWaveStatus {
    SwiftWaveStatus::NotInitialised // TODO (Phase 2)
}
