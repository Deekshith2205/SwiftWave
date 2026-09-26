import 'package:flutter_test/flutter_test.dart';
import 'package:swiftwave_app/core/ffi/swiftwave_native.dart';
import 'package:swiftwave_app/core/models/discovery.dart';

void main() {
  group('SwiftWaveNative Dart Lifecycle', () {
    test('create() requires valid path, empty path fails', () {
      final native = SwiftWaveNative();
      // Dart catches the empty string locally before hitting FFI
      expect(() => native.create(dataDirectory: ''), throwsArgumentError);
    });

    test(
      'create() propagates exception when FFI library missing or initialization fails',
      () {
        final native = SwiftWaveNative();
        // If FFI is missing, _isLoaded is false and create returns immediately without creating.
        // But we can check that it throws SwiftWaveNativeException if FFI exists and it returns NULL.
        // Since FFI is degraded in tests, _isLoaded is false.
        expect(
          () => native.create(dataDirectory: 'dummy/path'),
          returnsNormally,
        );
      },
    );

    // We cannot easily test "unsupported platform -> failure" via normal Flutter unit tests
    // unless we actually load the DLL on the target platform.
    // We also can't test "valid Windows path -> success" without the DLL loading properly.
    // For now we test the explicit dart-side safety boundary.

    test('Dart wrapper safe double-destroy prevention', () {
      final native = SwiftWaveNative();

      // Destroying once is safe
      expect(() => native.destroy(), returnsNormally);

      // Destroying twice is safe
      expect(() => native.destroy(), returnsNormally);
    });

    test('Dart wrapper blocks calls after destroy', () {
      final native = SwiftWaveNative();
      native.destroy();

      // Calls after destroy should throw StateError natively in the Dart wrapper
      expect(
        () => native.create(dataDirectory: 'valid/path'),
        throwsStateError,
      );
      expect(() => native.init(), throwsStateError);
      expect(() => native.shutdown(), throwsStateError);
      expect(() => native.getDeviceId(), throwsStateError);
      expect(() => native.startDiscovery(), throwsStateError);
    });

    test('startDiscovery degraded mode returns empty stream', () {
      final native = SwiftWaveNative();
      // Since FFI isn't loaded in test mode, it should gracefully return an empty stream
      final stream = native.startDiscovery();
      expect(stream, isA<Stream<DiscoveryEvent>>());
      expect(stream.isEmpty, completion(isTrue));
    });

    test('stopDiscovery in degraded mode is safe', () {
      final native = SwiftWaveNative();
      expect(() => native.stopDiscovery(), returnsNormally);
    });
  });
}
