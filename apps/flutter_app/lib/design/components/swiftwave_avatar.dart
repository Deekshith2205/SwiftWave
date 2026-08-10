import 'package:flutter/material.dart';

import '../colors.dart';

/// Avatar displaying device initials or an icon.
class SwiftWaveAvatar extends StatelessWidget {
  final String deviceName;
  final bool isTrusted;
  final double size;

  const SwiftWaveAvatar({
    super.key,
    required this.deviceName,
    this.isTrusted = false,
    this.size = 40,
  });

  String get _initials {
    if (deviceName.isEmpty) return '?';
    final parts = deviceName.trim().split(RegExp(r'\s+'));
    if (parts.length == 1) {
      return parts[0].substring(0, 1).toUpperCase();
    }
    return '${parts[0][0]}${parts[1][0]}'.toUpperCase();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final color = isTrusted ? SwiftWaveColors.statusSuccess : theme.colorScheme.primary;

    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        borderRadius: BorderRadius.circular(size * 0.3),
        border: Border.all(color: color.withAlpha(60)),
      ),
      alignment: Alignment.center,
      child: Text(
        _initials,
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.w700,
          fontSize: size * 0.4,
        ),
      ),
    );
  }
}
