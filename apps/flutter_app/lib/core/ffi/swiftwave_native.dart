import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';

// ---------------------------------------------------------------------------
// Native function typedefs
// ---------------------------------------------------------------------------

final class SwiftWaveHandle extends Opaque {}

typedef _SwiftWaveCreateNative = Pointer<SwiftWaveHandle> Function(Pointer<Utf8>);
typedef _SwiftWaveCreateDart = Pointer<SwiftWaveHandle> Function(Pointer<Utf8>);

typedef _SwiftWaveDestroyNative = Void Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveDestroyDart = void Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveInitNative = Int32 Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveInitDart = int Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveShutdownNative = Int32 Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveShutdownDart = int Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveGetDeviceIdNative = Pointer<Utf8> Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveGetDeviceIdDart = Pointer<Utf8> Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveFreeStringNative = Void Function(Pointer<Utf8>);
typedef _SwiftWaveFreeStringDart = void Function(Pointer<Utf8>);

typedef _SwiftWaveVersionNative = Pointer<Utf8> Function();
typedef _SwiftWaveVersionDart = Pointer<Utf8> Function();

// ---------------------------------------------------------------------------
// Bridge class
// ---------------------------------------------------------------------------

class SwiftWaveNativeException implements Exception {
  final int statusCode;
  final String message;
  SwiftWaveNativeException(this.statusCode, this.message);

  @override
  String toString() => 'SwiftWaveNativeException($statusCode): $message';
}

/// Strongly-typed FFI bridge to the `swiftwave_ffi` Rust library.
/// Each instance conceptually owns at most one `SwiftWaveHandle`.
class SwiftWaveNative {
  SwiftWaveNative() {
    _load();
  }

  static DynamicLibrary? _lib;
  Pointer<SwiftWaveHandle> _handle = nullptr;

  static late final _SwiftWaveCreateDart _create;
  static late final _SwiftWaveDestroyDart _destroy;
  static late final _SwiftWaveInitDart _init;
  static late final _SwiftWaveShutdownDart _shutdown;
  static late final _SwiftWaveGetDeviceIdDart _getDeviceId;
  static late final _SwiftWaveFreeStringDart _freeString;
  static late final _SwiftWaveVersionDart _version;

  static bool _isLoaded = false;
  bool _isDestroyed = false;

  /// Resolve the platform-specific library path.
  static String _libraryPath() {
    if (Platform.isWindows) return 'swiftwave_ffi.dll';
    if (Platform.isLinux || Platform.isAndroid) return 'libswiftwave_ffi.so';
    if (Platform.isMacOS) return 'libswiftwave_ffi.dylib';
    throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
  }

  /// Load the library and lookup symbols globally.
  static void _load() {
    if (_isLoaded) return;
    
    // Fallback gracefully if FFI lib isn't built yet (e.g. during UI-only tests)
    try {
      _lib = DynamicLibrary.open(_libraryPath());
    } catch (e) {
      print('[SwiftWaveNative] FFI library not found. Running in degraded mode. Error: $e');
      return;
    }

    final lib = _lib!;
    _create = lib.lookupFunction<_SwiftWaveCreateNative, _SwiftWaveCreateDart>('swiftwave_create');
    _destroy = lib.lookupFunction<_SwiftWaveDestroyNative, _SwiftWaveDestroyDart>('swiftwave_destroy');
    _init = lib.lookupFunction<_SwiftWaveInitNative, _SwiftWaveInitDart>('swiftwave_init');
    _shutdown = lib.lookupFunction<_SwiftWaveShutdownNative, _SwiftWaveShutdownDart>('swiftwave_shutdown');
    _getDeviceId = lib.lookupFunction<_SwiftWaveGetDeviceIdNative, _SwiftWaveGetDeviceIdDart>('swiftwave_get_device_id');
    _freeString = lib.lookupFunction<_SwiftWaveFreeStringNative, _SwiftWaveFreeStringDart>('swiftwave_free_string');
    _version = lib.lookupFunction<_SwiftWaveVersionNative, _SwiftWaveVersionDart>('swiftwave_version');

    _isLoaded = true;
  }

  /// Create the runtime instance backed by secure persistent storage.
  /// [dataDirectory] is required.
  void create({required String dataDirectory}) {
    if (dataDirectory.isEmpty) {
      throw ArgumentError('dataDirectory cannot be empty');
    }

    if (_isDestroyed) throw StateError('Handle has been destroyed');
    if (!_isLoaded) return;
    if (_handle != nullptr) return; // already created

    Pointer<Utf8> cPath = dataDirectory.toNativeUtf8();

    try {
      _handle = _create(cPath);
      if (_handle == nullptr) {
        throw SwiftWaveNativeException(5, 'Failed to create SwiftWave runtime');
      }
    } finally {
      if (cPath != nullptr) {
        malloc.free(cPath);
      }
    }
  }

  /// Initialize the runtime components.
  void init() {
    if (_isDestroyed) throw StateError('Handle has been destroyed');
    if (!_isLoaded) return;
    if (_handle == nullptr) throw SwiftWaveNativeException(1, 'Handle is null');

    final status = _init(_handle);
    if (status != 0) {
      throw SwiftWaveNativeException(status, 'Failed to initialize runtime');
    }
  }

  /// Shutdown the runtime gracefully.
  void shutdown() {
    if (_isDestroyed) throw StateError('Handle has been destroyed');
    if (!_isLoaded) return;
    if (_handle == nullptr) return;

    final status = _shutdown(_handle);
    if (status != 0 && status != 9) { // 9 = AlreadyShutdown
      print('[SwiftWaveNative] Warning: shutdown returned status $status');
    }
  }

  /// Destroy the runtime and free memory.
  /// Calling this multiple times is safe at the Dart layer.
  void destroy() {
    if (_isDestroyed) return; // double destroy prevention
    if (!_isLoaded) {
      _isDestroyed = true;
      return;
    }
    
    if (_handle != nullptr) {
      _destroy(_handle);
      _handle = nullptr;
    }
    _isDestroyed = true;
  }

  /// Get the current version of the native library.
  String getVersion() {
    if (!_isLoaded) return '0.0.0-stub';
    
    final ptr = _version();
    if (ptr == nullptr) return 'unknown';
    return ptr.toDartString();
  }

  /// Get the device ID string.
  String? getDeviceId() {
    if (_isDestroyed) throw StateError('Handle has been destroyed');
    if (!_isLoaded) return '00000000-0000-0000-0000-000000000000';
    if (_handle == nullptr) return null;

    final ptr = _getDeviceId(_handle);
    if (ptr == nullptr) return null;

    final str = ptr.toDartString();
    _freeString(ptr);
    return str;
  }
}
