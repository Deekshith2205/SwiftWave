//! swiftwave_ffi — C-ABI boundary between Dart FFI and the Rust core.
//!
//! This crate exposes a stable `extern "C"` API. It is intentionally thin:
//! - No business logic lives here.
//! - All real work is delegated to `swiftwave_core`.
//! - All strings cross the boundary as null-terminated UTF-8 (`*const c_char`).
//! - Ownership is documented on every function.
//!
//! # Safety
//! Pointer arguments MUST be valid, non-null, and properly aligned.
//! The caller (Dart side) is responsible for memory management of
//! strings returned by this library (use `swiftwave_ffi_free_string`).
//!
//! # TODO (Phase 2)
//! - Generate `swiftwave_ffi.h` automatically via `cbindgen`.
//! - Add async callback registration (progress updates, incoming transfers).
//! - Replace raw pointers with a handle-based API for safer Dart interop.

pub mod api;

// Re-export all public API symbols so the linker can find them.
pub use api::*;

#[cfg(test)]
mod tests;
