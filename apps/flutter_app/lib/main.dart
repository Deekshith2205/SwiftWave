import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'app/app.dart';
import 'core/ffi/swiftwave_native.dart';

/// Entry point for SwiftWave.
void main() {
  // Ensure Flutter engine is initialised before any plugin/FFI calls.
  WidgetsFlutterBinding.ensureInitialized();

  // The application relies on swiftWaveRuntimeProvider to eagerly or lazily
  // bootstrap the SwiftWave runtime.


  runApp(
    const ProviderScope(
      child: SwiftWaveApp(),
    ),
  );
}
