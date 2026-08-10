import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../design/theme.dart';
import 'router.dart';

/// Root application widget.
///
/// Configures Material 3 theming using [SwiftWaveTheme].
class SwiftWaveApp extends ConsumerWidget {
  const SwiftWaveApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);

    return MaterialApp.router(
      title: 'SwiftWave',
      debugShowCheckedModeBanner: false,
      routerConfig: router,
      theme: SwiftWaveTheme.lightTheme,
      darkTheme: SwiftWaveTheme.darkTheme,
      themeMode: ThemeMode.system, // TODO: Read from settings
    );
  }
}
