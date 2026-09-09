import 'package:flutter_test/flutter_test.dart';
import 'package:swiftwave_app/core/ffi/swiftwave_native.dart';

void main() {
  group('SwiftWaveNative Dart Lifecycle', () {
    test('Initialization degrades gracefully when FFI library is missing', () {
      final native = SwiftWaveNative();
      
      // In a Flutter test environment without the .so/.dll, constructor will catch the exception
      // and put it in degraded mode without crashing.
      
      // These should return immediately if not loaded, without throwing exceptions
      expect(() => native.create(), returnsNormally);
      expect(() => native.init(), returnsNormally);
      
      // version should return stub
      expect(native.getVersion(), '0.0.0-stub');
      
      // getDeviceId should return default mock UUID
      expect(native.getDeviceId(), '00000000-0000-0000-0000-000000000000');
      
      expect(() => native.shutdown(), returnsNormally);
      expect(() => native.destroy(), returnsNormally);
    });

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
      expect(() => native.create(), throwsStateError);
      expect(() => native.init(), throwsStateError);
      expect(() => native.shutdown(), throwsStateError);
      expect(() => native.getDeviceId(), throwsStateError);
      
      // Version is static, doesn't need handle, so it doesn't throw StateError
      expect(native.getVersion(), '0.0.0-stub');
    });

    test('Dart wrapper isolates independent handles', () {
      final native1 = SwiftWaveNative();
      final native2 = SwiftWaveNative();
      
      expect(native1, isNot(equals(native2)));
      
      native1.destroy();
      
      // native1 is destroyed
      expect(() => native1.init(), throwsStateError);
      
      // native2 is NOT destroyed
      expect(() => native2.init(), returnsNormally);
      
      native2.destroy();
    });
  });
}
