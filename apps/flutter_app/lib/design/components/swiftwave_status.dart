import 'package:flutter/material.dart';

import '../colors.dart';

enum SwiftWaveStatusType { online, offline, error, warning }

/// Small pill indicator for connection or general status.
class SwiftWaveStatusBadge extends StatelessWidget {
  final SwiftWaveStatusType type;
  final String label;

  const SwiftWaveStatusBadge({
    super.key,
    required this.type,
    required this.label,
  });

  Color _getColor(BuildContext context) {
    switch (type) {
      case SwiftWaveStatusType.online:
        return SwiftWaveColors.statusSuccess;
      case SwiftWaveStatusType.offline:
        return Theme.of(context).colorScheme.onSurface.withAlpha(150);
      case SwiftWaveStatusType.error:
        return SwiftWaveColors.statusError;
      case SwiftWaveStatusType.warning:
        return SwiftWaveColors.statusWarning;
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = _getColor(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: color.withAlpha(60)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 6,
            height: 6,
            decoration: BoxDecoration(color: color, shape: BoxShape.circle),
          ),
          const SizedBox(width: 6),
          Text(
            label,
            style: TextStyle(
              fontSize: 11,
              fontWeight: FontWeight.w600,
              color: color,
            ),
          ),
        ],
      ),
    );
  }
}
