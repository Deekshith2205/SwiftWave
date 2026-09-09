# FFI Architecture & Memory Safety

This document describes the C ABI boundary bridging the Flutter/Dart application with the Rust `swiftwave_core` engine.

## Architecture & Ownership

The SwiftWave FFI uses a strict opaque handle-based architecture to isolate Rust application state from the Dart garbage collector and the Dart isolate thread. 

Flutter does not manage global state. Instead, the Flutter app holds a raw C pointer (`*mut SwiftWaveHandle`) linking to a heap-allocated Rust runtime (`SwiftWaveRuntime`).

```text
  Flutter App (Dart Isolate)
       │
       ▼
  SwiftWaveNative (Dart Wrapper)
       │
       ▼
  C ABI (Opaque Handle)
       │
       ▼
  SwiftWaveHandle (Box<SwiftWaveRuntime>)
       │
       ▼
  swiftwave_core (Tokio RT, Lifecycle State)
```

### Opaque Handles
`SwiftWaveHandle` is an opaque pointer (`Opaque` in Dart, `Box<SwiftWaveRuntime>` in Rust). Dart never dereferences this pointer and does not understand its memory layout. The C ABI strictly enforces pointers.

## Memory Safety Guarantees

1. **Null Pointer Checks**: Every FFI function performs an `is_null()` validation on the incoming handle. Passing a null pointer will safely return `SwiftWaveStatus::NullPointer`.
2. **Invalid/Dangling Pointers**: Arbitrary invalid or dangling pointers are **NOT** safely detectable. Handles must originate from `swiftwave_create`. Destroying or using a handle after destruction is **caller-side Undefined Behavior**.
3. **Panic Containment**: Operations cross the boundary returning a unified `SwiftWaveStatus`. Top-level exported FFI functions are wrapped in `std::panic::catch_unwind` to structurally guarantee no Rust panic unwinds into Dart. Panics are mapped to `SwiftWaveStatus::InternalError` or `null` safely.
4. **String Allocation & Ownership**:
   - Native strings passed to Dart are allocated via `CString::into_raw`.
   - Dart MUST pass them back to `swiftwave_free_string` for deallocation. 

## Thread Safety

`SwiftWaveRuntime` protects internal states using `RwLock`. Dart can call `swiftwave_init`, `swiftwave_get_device_id`, and `swiftwave_shutdown` from multiple isolates if necessary.
However, **`swiftwave_destroy` requires exclusive external synchronization** against all other calls using that handle to prevent use-after-free races.

## Runtime Lifecycle

The runtime transitions linearly through lifecycle states:

1. **Created**: Handle allocated. Resources uninitialized.
2. **Initialized**: Tokio active, CoreConfig loaded, Identity loaded.
3. **Shutdown**: All resources cleared. Further initializations blocked.

## API Reference

| API | Purpose | Ownership | Thread-safe |
|---|---|---|---|
| `swiftwave_create` | Allocates the runtime. | Rust-allocated, Dart-owned. | Yes |
| `swiftwave_init` | Starts services and identity. | Borrows handle. | Yes |
| `swiftwave_shutdown` | Stops services. | Borrows handle. | Yes |
| `swiftwave_destroy` | Deallocates runtime. | Rust-destroyed. | **No** (Exclusive external sync required) |
| `swiftwave_get_device_id` | Queries the device fingerprint. | Rust-allocated, Dart-owned. | Yes |
| `swiftwave_free_string` | Frees an FFI C-string. | Rust-destroyed. | Yes |
| `swiftwave_version` | Queries library version. | Static lifetime. | Yes |

## Error Model

`SwiftWaveStatus` enum maps uniformly to Dart `int`.

- `0` - Success
- `1` - InvalidHandle
- `2` - NotInitialized
- `3` - AlreadyInitialized
- `4` - IoError
- `5` - InternalError
- `6` - NullPointer
- `7` - InvalidUtf8
- `8` - Shutdown
- `9` - AlreadyShutdown

## Future Extension Strategy (Phase 2+)

In Phase 2, `swiftwave_init` will ingest OS paths and the actual DeviceIdentity keystore callbacks. FFI stubs for Discovery and File Transfer will be expanded to take asynchronous Dart callbacks (ports) for UI updates.
