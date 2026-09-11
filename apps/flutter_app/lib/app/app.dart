import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../design/theme.dart';
import 'router.dart';

import '../core/providers/runtime_provider.dart';

/// Root application widget.
///
/// Configures Material 3 theming using [SwiftWaveTheme].
class SwiftWaveApp extends ConsumerWidget {
  const SwiftWaveApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);
    final runtimeAsync = ref.watch(swiftWaveRuntimeProvider);

    return MaterialApp.router(
      title: 'SwiftWave',
      debugShowCheckedModeBanner: false,
      routerConfig: router,
      theme: SwiftWaveTheme.lightTheme,
      darkTheme: SwiftWaveTheme.darkTheme,
      themeMode: ThemeMode.system, // TODO: Read from settings
      builder: (context, child) {
        return runtimeAsync.when(
          data: (_) => child!,
          loading: () => const Scaffold(
            body: Center(
              child: CircularProgressIndicator(),
            ),
          ),
          error: (err, stack) => Scaffold(
            body: Center(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(Icons.error_outline, color: Colors.red, size: 48),
                    const SizedBox(height: 16),
                    Text(
                      'Failed to initialize secure storage.\n\nError: $err',
                      textAlign: TextAlign.center,
                      style: const TextStyle(color: Colors.red),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

