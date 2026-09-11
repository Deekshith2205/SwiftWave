import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import '../ffi/swiftwave_native.dart';
import 'package:flutter/foundation.dart';

/// Exposes the global lifecycle of the SwiftWave native runtime asynchronously.
final swiftWaveRuntimeProvider = FutureProvider<SwiftWaveNative>((ref) async {
  final runtime = SwiftWaveNative();

  try {
    // Phase 2D: Production secure storage integration
    // Resolves the persistent application support directory.
    final directory = await getApplicationSupportDirectory();
    
    // Creates the runtime using the production path. 
    runtime.create(dataDirectory: directory.path);
    runtime.init();
  } catch (e, stackTrace) {
    debugPrint('[SwiftWave] Failed to initialize native runtime: $e\n$stackTrace');
    // We do NOT fall back to fake success / degraded mock in production if secure storage fails.
    // Propagate the error.
    rethrow;
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
