import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'app/app.dart';

/// Entry point for SwiftWave.
///
/// Wraps the widget tree in [ProviderScope] so that all Riverpod providers
/// are accessible throughout the app.
void main() {
  // Ensure Flutter engine is initialised before any plugin/FFI calls.
  WidgetsFlutterBinding.ensureInitialized();

  // TODO (Phase 2): call swiftwave_ffi_init() here via the FFI bridge.

  runApp(
    const ProviderScope(
      child: SwiftWaveApp(),
    ),
  );
}
