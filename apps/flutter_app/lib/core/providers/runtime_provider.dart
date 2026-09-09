import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../ffi/swiftwave_native.dart';
import 'package:flutter/foundation.dart';

/// Exposes the global lifecycle of the SwiftWave native runtime.
final swiftWaveRuntimeProvider = Provider<SwiftWaveNative>((ref) {
  final runtime = SwiftWaveNative();
  
  try {
    runtime.create();
    runtime.init();
  } catch (e, stackTrace) {
    debugPrint('[SwiftWave] Failed to initialize native runtime: $e\n$stackTrace');
    // For Phase 1, we continue running gracefully in degraded mode if FFI fails.
  }

  // Clean up native resources when the provider is destroyed.
  //
  // ANDROID LIFECYCLE LIMITATION:
  // Provider disposal normally performs shutdown() and destroy().
  // However, Android may terminate the application process abruptly in the background
  // without triggering `ref.onDispose`.
  // Application correctness and security MUST NOT depend on `onDispose` firing during
  // process termination. Native runtime drops the global state naturally when the process dies.
  // Phase 2+ persistence/recovery mechanisms must be designed independently of this callback.
  ref.onDispose(() {
    try {
      runtime.shutdown();
    } catch (_) {}
    runtime.destroy();
  });

  return runtime;
});
