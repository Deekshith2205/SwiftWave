import 'dart:io';

/// Dart FFI bridge to the Rust `swiftwave_ffi` cdylib.
///
/// # Phase 1
/// This file is a skeleton with documented bindings but no working
/// FFI calls. All methods throw [UnsupportedError] until Phase 2.
///
/// # Phase 2 Plan
/// 1. Run `cargo build --release` to produce:
///    - `swiftwave_ffi.dll` (Windows)
///    - `libswiftwave_ffi.so` (Linux / Android)
///    - `libswiftwave_ffi.dylib` (macOS)
/// 2. Run `ffigen` against `swiftwave_ffi.h` to auto-generate typed bindings.
/// 3. Copy the generated file here (or import it as a package).
/// 4. Replace the stub calls below with real FFI invocations.
///
/// # Loading the library
/// ```dart
/// final lib = SwiftWaveFfi.instance;
/// lib.init();
/// ```

// ---------------------------------------------------------------------------
// Native function typedefs
// ---------------------------------------------------------------------------

// TODO (Phase 2): auto-generate these from swiftwave_ffi.h via ffigen.

/// `SwiftWaveStatus swiftwave_ffi_init()`
// typedef _SwiftWaveFfiInitNative = Int32 Function();
// typedef _SwiftWaveFfiInitDart = int Function();

/// `void swiftwave_ffi_shutdown()`
// typedef _SwiftWaveFfiShutdownNative = Void Function();
// typedef _SwiftWaveFfiShutdownDart = void Function();

/// `char* swiftwave_ffi_generate_device_id()`
// typedef _GenerateDeviceIdNative = Pointer<Utf8> Function();
// typedef _GenerateDeviceIdDart = Pointer<Utf8> Function();

// ---------------------------------------------------------------------------
// Bridge class
// ---------------------------------------------------------------------------

/// Singleton FFI bridge to the `swiftwave_ffi` Rust library.
class SwiftWaveFfi {
  SwiftWaveFfi._();

  static final SwiftWaveFfi _instance = SwiftWaveFfi._();

  /// The singleton instance.
  static SwiftWaveFfi get instance => _instance;

  // TODO (Phase 2): load the library and look up symbols.
  // DynamicLibrary? _lib;

  /// Resolve the platform-specific library path.
  static String _libraryPath() {
    if (Platform.isWindows) return 'swiftwave_ffi.dll';
    if (Platform.isLinux || Platform.isAndroid) return 'libswiftwave_ffi.so';
    if (Platform.isMacOS) return 'libswiftwave_ffi.dylib';
    throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
  }

  // ---------------------------------------------------------------------------
  // API stubs — TODO (Phase 2): replace with real FFI calls
  // ---------------------------------------------------------------------------

  /// Initialise the Rust swiftwave_ffi library.
  ///
  /// Must be called once before any other method.
  ///
  /// TODO (Phase 2): call `swiftwave_ffi_init()` via FFI.
  void init() {
    // ignore: avoid_print
    print('[SwiftWaveFfi] init() — stub (Phase 2 TODO). Library: ${_libraryPath()}');
  }

  /// Shut down the Rust library and release all resources.
  ///
  /// TODO (Phase 2): call `swiftwave_ffi_shutdown()` via FFI.
  void shutdown() {
    // TODO (Phase 2): implement
  }

  /// Generate a new device ID from the Rust identity module.
  ///
  /// Returns a placeholder UUID string until Phase 2.
  ///
  /// TODO (Phase 2): call `swiftwave_ffi_generate_device_id()` via FFI.
  String generateDeviceId() {
    // TODO (Phase 2): call real FFI function and free returned C string.
    return '00000000-0000-0000-0000-000000000000';
  }

  /// Start peer discovery.
  ///
  /// TODO (Phase 2): call `swiftwave_ffi_start_discovery()` + register callback.
  void startDiscovery() {
    // TODO (Phase 2): implement
    throw UnimplementedError('Discovery not yet implemented (Phase 2)');
  }

  /// Stop peer discovery.
  ///
  /// TODO (Phase 2): call `swiftwave_ffi_stop_discovery()`.
  void stopDiscovery() {
    // TODO (Phase 2): implement
  }
}
