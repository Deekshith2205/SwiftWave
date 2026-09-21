import 'dart:ffi';
import 'dart:io';
import 'dart:convert';
import 'dart:async';
import 'package:ffi/ffi.dart';
import '../models/discovery.dart';
import 'package:flutter/foundation.dart';

// ---------------------------------------------------------------------------
// Native function typedefs
// ---------------------------------------------------------------------------

final class SwiftWaveHandle extends Opaque {}

typedef _SwiftWaveCreateNative =
    Pointer<SwiftWaveHandle> Function(Pointer<Utf8>);
typedef _SwiftWaveCreateDart = Pointer<SwiftWaveHandle> Function(Pointer<Utf8>);

typedef _SwiftWaveDestroyNative = Void Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveDestroyDart = void Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveInitNative = Int32 Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveInitDart = int Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveShutdownNative = Int32 Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveShutdownDart = int Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveGetDeviceIdNative =
    Pointer<Utf8> Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveGetDeviceIdDart =
    Pointer<Utf8> Function(Pointer<SwiftWaveHandle>);

typedef _SwiftWaveFreeStringNative = Void Function(Pointer<Utf8>);
typedef _SwiftWaveFreeStringDart = void Function(Pointer<Utf8>);

typedef _SwiftWaveStartDiscoveryNative =
    Int32 Function(
      Pointer<SwiftWaveHandle>,
      Uint16,
      Pointer<NativeFunction<Void Function(CDiscoveryEvent)>>,
    );
typedef _SwiftWaveStartDiscoveryDart =
    int Function(
      Pointer<SwiftWaveHandle>,
      int,
      Pointer<NativeFunction<Void Function(CDiscoveryEvent)>>,
    );

typedef _SwiftWaveStopDiscoveryNative =
    Int32 Function(Pointer<SwiftWaveHandle>);
typedef _SwiftWaveStopDiscoveryDart = int Function(Pointer<SwiftWaveHandle>);

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
  static late final _SwiftWaveStartDiscoveryDart _startDiscovery;
  static late final _SwiftWaveStopDiscoveryDart _stopDiscovery;
  static late final _SwiftWaveVersionDart _version;

  NativeCallable<Void Function(CDiscoveryEvent)>? _discoveryCallable;
  StreamController<DiscoveryEvent>? _discoveryStreamController;

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
      debugPrint(
        '[SwiftWaveNative] FFI library not found. Running in degraded mode. Error: $e',
      );
      return;
    }

    final lib = _lib!;
    _create = lib.lookupFunction<_SwiftWaveCreateNative, _SwiftWaveCreateDart>(
      'swiftwave_create',
    );
    _destroy = lib
        .lookupFunction<_SwiftWaveDestroyNative, _SwiftWaveDestroyDart>(
          'swiftwave_destroy',
        );
    _init = lib.lookupFunction<_SwiftWaveInitNative, _SwiftWaveInitDart>(
      'swiftwave_init',
    );
    _shutdown = lib
        .lookupFunction<_SwiftWaveShutdownNative, _SwiftWaveShutdownDart>(
          'swiftwave_shutdown',
        );
    _getDeviceId = lib
        .lookupFunction<_SwiftWaveGetDeviceIdNative, _SwiftWaveGetDeviceIdDart>(
          'swiftwave_get_device_id',
        );
    _freeString = lib
        .lookupFunction<_SwiftWaveFreeStringNative, _SwiftWaveFreeStringDart>(
          'swiftwave_free_string',
        );
    _startDiscovery = lib
        .lookupFunction<
          _SwiftWaveStartDiscoveryNative,
          _SwiftWaveStartDiscoveryDart
        >('swiftwave_start_discovery');
    _stopDiscovery = lib
        .lookupFunction<
          _SwiftWaveStopDiscoveryNative,
          _SwiftWaveStopDiscoveryDart
        >('swiftwave_stop_discovery');
    _version = lib
        .lookupFunction<_SwiftWaveVersionNative, _SwiftWaveVersionDart>(
          'swiftwave_version',
        );

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
    if (status != 0 && status != 9) {
      // 9 = AlreadyShutdown
      debugPrint('[SwiftWaveNative] Warning: shutdown returned status $status');
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

    stopDiscovery();

    if (_handle != nullptr) {
      _destroy(_handle);
      _handle = nullptr;
    }
    _isDestroyed = true;
  }

  String _decodeCArray(Array<Int8> arr, int maxLen) {
    final bytes = <int>[];
    for (int i = 0; i < maxLen; i++) {
      final b = arr[i];
      if (b == 0) break;
      bytes.add(b);
    }
    return utf8.decode(bytes);
  }

  /// Get the current version of the native library.
  String getVersion() {
    if (!_isLoaded) return '0.0.0-stub';
    final ptr = _version();
    return ptr.toDartString();
  }

  /// Start mDNS discovery on the local network.
  /// Returns a stream of [DiscoveryEvent].
  Stream<DiscoveryEvent> startDiscovery({required int quicPort}) {
    if (_isDestroyed) throw StateError('Handle is destroyed');
    if (!_isLoaded) return const Stream.empty();

    if (_discoveryStreamController != null) {
      return _discoveryStreamController!.stream;
    }

    _discoveryStreamController = StreamController<DiscoveryEvent>.broadcast(
      onCancel: () {
        stopDiscovery();
      },
    );

    _discoveryCallable =
        NativeCallable<Void Function(CDiscoveryEvent)>.listener((
          CDiscoveryEvent event,
        ) {
          if (event.eventType == 0) {
            // PeerFound
            final peer = DiscoveredPeer(
              fingerprint: _decodeCArray(event.fingerprint, 65),
              displayName: _decodeCArray(event.displayName, 65),
              address: _decodeCArray(event.address, 65),
              medium: DiscoveryMedium.values[event.medium],
              rssi: event.rssiHasValue == 1 ? event.rssi : null,
              protocolVersion: event.protocolVersion,
              lastSeen: event.lastSeen,
            );
            _discoveryStreamController?.add(DiscoveryEvent.peerFound(peer));
          } else if (event.eventType == 1) {
            // PeerLost
            final fp = _decodeCArray(event.fingerprint, 65);
            _discoveryStreamController?.add(DiscoveryEvent.peerLost(fp));
          }
        });

    final status = _startDiscovery(
      _handle,
      quicPort,
      _discoveryCallable!.nativeFunction,
    );
    if (status != 0) {
      _discoveryCallable?.close();
      _discoveryCallable = null;
      throw Exception('startDiscovery failed with status $status');
    }

    return _discoveryStreamController!.stream;
  }

  /// Stop mDNS discovery.
  void stopDiscovery() {
    if (!_isLoaded || _isDestroyed) return;

    if (_discoveryCallable != null) {
      _stopDiscovery(_handle);
      _discoveryCallable?.close();
      _discoveryCallable = null;
    }

    if (_discoveryStreamController != null) {
      _discoveryStreamController?.close();
      _discoveryStreamController = null;
    }
  }

  /// Get the device ID string.
  String? getDeviceId() {
    if (_isDestroyed) throw StateError('Handle has been destroyed');
    if (!_isLoaded) return null;
    final ptr = _getDeviceId(_handle);
    if (ptr == nullptr) return null;

    final id = ptr.toDartString();
    _freeString(ptr);
    return id;
  }
}

// ---------------------------------------------------------------------------
// Native FFI Structures
// ---------------------------------------------------------------------------

final class CDiscoveryEvent extends Struct {
  @Uint8()
  external int eventType;

  @Array(65)
  external Array<Int8> fingerprint;

  @Array(65)
  external Array<Int8> displayName;

  @Array(65)
  external Array<Int8> address;

  @Uint8()
  external int medium;

  @Uint8()
  external int rssiHasValue;

  @Int8()
  external int rssi;

  @Uint16()
  external int protocolVersion;

  @Uint64()
  external int lastSeen;
}
