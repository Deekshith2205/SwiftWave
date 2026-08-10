import 'package:flutter/material.dart';

/// Semantic color tokens for the SwiftWave design system.
///
/// Designed to feel premium, minimal, and native.
abstract final class SwiftWaveColors {
  // ---------------------------------------------------------------------------
  // Accents (Primary / Teal)
  // ---------------------------------------------------------------------------
  static const Color accent = Color(0xFF00D4AA); // Primary brand color (Dark theme default)
  static const Color accentDark = Color(0xFF00A884); // Primary brand color (Light theme default)
  
  static const Color accentMuted = Color(0x3300D4AA); // 20% opacity for backgrounds

  // ---------------------------------------------------------------------------
  // Dark Theme Surfaces
  // ---------------------------------------------------------------------------
  static const Color darkSurface = Color(0xFF0D1117); // Deep background (GitHub Dark Dimmed style)
  static const Color darkSurfaceElevated = Color(0xFF161B22); // Card background
  static const Color darkSurfaceHighlight = Color(0xFF21262D); // Hover/pressed states
  static const Color darkOutline = Color(0xFF30363D); // Subtle borders

  static const Color darkTextHigh = Color(0xFFF0F6FC); // Primary text
  static const Color darkTextMedium = Color(0xFF8B949E); // Secondary text
  static const Color darkTextLow = Color(0xFF484F58); // Disabled/ghost text

  // ---------------------------------------------------------------------------
  // Light Theme Surfaces
  // ---------------------------------------------------------------------------
  static const Color lightSurface = Color(0xFFF6F8FA);
  static const Color lightSurfaceElevated = Color(0xFFFFFFFF);
  static const Color lightSurfaceHighlight = Color(0xFFEAECEF);
  static const Color lightOutline = Color(0xFFD0D7DE);

  static const Color lightTextHigh = Color(0xFF1F2328);
  static const Color lightTextMedium = Color(0xFF656D76);
  static const Color lightTextLow = Color(0xFF8C959F);

  // ---------------------------------------------------------------------------
  // Status Colors
  // ---------------------------------------------------------------------------
  static const Color statusSuccess = Color(0xFF238636);
  static const Color statusWarning = Color(0xFFD29922);
  static const Color statusError = Color(0xFFF85149);
}
