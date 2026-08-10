import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

import 'colors.dart';

/// Global application theme configuration for SwiftWave.
abstract final class SwiftWaveTheme {
  static ThemeData get darkTheme {
    final textTheme = GoogleFonts.interTextTheme(
      ThemeData(brightness: Brightness.dark).textTheme,
    );

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: const ColorScheme.dark(
        primary: SwiftWaveColors.accent,
        surface: SwiftWaveColors.darkSurface,
        surfaceContainerHighest: SwiftWaveColors.darkSurfaceElevated,
        onSurface: SwiftWaveColors.darkTextHigh,
        onSurfaceVariant: SwiftWaveColors.darkTextMedium,
        outline: SwiftWaveColors.darkOutline,
      ),
      scaffoldBackgroundColor: SwiftWaveColors.darkSurface,
      textTheme: textTheme.copyWith(
        titleLarge: textTheme.titleLarge?.copyWith(fontWeight: FontWeight.w600, letterSpacing: -0.5),
        titleMedium: textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
        bodyLarge: textTheme.bodyLarge?.copyWith(color: SwiftWaveColors.darkTextHigh),
        bodyMedium: textTheme.bodyMedium?.copyWith(color: SwiftWaveColors.darkTextMedium),
      ),
      appBarTheme: AppBarTheme(
        backgroundColor: SwiftWaveColors.darkSurface,
        foregroundColor: SwiftWaveColors.darkTextHigh,
        elevation: 0,
        centerTitle: false,
        scrolledUnderElevation: 0,
        titleTextStyle: textTheme.titleLarge?.copyWith(
          fontSize: 20,
          fontWeight: FontWeight.w600,
          color: SwiftWaveColors.darkTextHigh,
        ),
        iconTheme: const IconThemeData(color: SwiftWaveColors.darkTextHigh),
      ),
      navigationRailTheme: const NavigationRailThemeData(
        backgroundColor: SwiftWaveColors.darkSurfaceElevated,
        indicatorColor: SwiftWaveColors.accentMuted,
        selectedIconTheme: IconThemeData(color: SwiftWaveColors.accent),
        unselectedIconTheme: IconThemeData(color: SwiftWaveColors.darkTextMedium),
      ),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: SwiftWaveColors.darkSurfaceElevated,
        indicatorColor: SwiftWaveColors.accentMuted,
        iconTheme: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return const IconThemeData(color: SwiftWaveColors.accent);
          }
          return const IconThemeData(color: SwiftWaveColors.darkTextMedium);
        }),
      ),
      dividerTheme: const DividerThemeData(
        color: SwiftWaveColors.darkOutline,
        thickness: 1,
        space: 1,
      ),
    );
  }

  static ThemeData get lightTheme {
    final textTheme = GoogleFonts.interTextTheme(
      ThemeData(brightness: Brightness.light).textTheme,
    );

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      colorScheme: const ColorScheme.light(
        primary: SwiftWaveColors.accentDark,
        surface: SwiftWaveColors.lightSurface,
        surfaceContainerHighest: SwiftWaveColors.lightSurfaceElevated,
        onSurface: SwiftWaveColors.lightTextHigh,
        onSurfaceVariant: SwiftWaveColors.lightTextMedium,
        outline: SwiftWaveColors.lightOutline,
      ),
      scaffoldBackgroundColor: SwiftWaveColors.lightSurface,
      textTheme: textTheme.copyWith(
        titleLarge: textTheme.titleLarge?.copyWith(fontWeight: FontWeight.w600, letterSpacing: -0.5),
        titleMedium: textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
        bodyLarge: textTheme.bodyLarge?.copyWith(color: SwiftWaveColors.lightTextHigh),
        bodyMedium: textTheme.bodyMedium?.copyWith(color: SwiftWaveColors.lightTextMedium),
      ),
      appBarTheme: AppBarTheme(
        backgroundColor: SwiftWaveColors.lightSurface,
        foregroundColor: SwiftWaveColors.lightTextHigh,
        elevation: 0,
        centerTitle: false,
        scrolledUnderElevation: 0,
        titleTextStyle: textTheme.titleLarge?.copyWith(
          fontSize: 20,
          fontWeight: FontWeight.w600,
          color: SwiftWaveColors.lightTextHigh,
        ),
        iconTheme: const IconThemeData(color: SwiftWaveColors.lightTextHigh),
      ),
      navigationRailTheme: const NavigationRailThemeData(
        backgroundColor: SwiftWaveColors.lightSurfaceElevated,
        indicatorColor: SwiftWaveColors.accentMuted, // Uses alpha, looks fine in light mode
        selectedIconTheme: IconThemeData(color: SwiftWaveColors.accentDark),
        unselectedIconTheme: IconThemeData(color: SwiftWaveColors.lightTextMedium),
      ),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: SwiftWaveColors.lightSurfaceElevated,
        indicatorColor: SwiftWaveColors.accentMuted,
        iconTheme: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return const IconThemeData(color: SwiftWaveColors.accentDark);
          }
          return const IconThemeData(color: SwiftWaveColors.lightTextMedium);
        }),
      ),
      dividerTheme: const DividerThemeData(
        color: SwiftWaveColors.lightOutline,
        thickness: 1,
        space: 1,
      ),
    );
  }
}
