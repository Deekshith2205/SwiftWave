import 'package:flutter/material.dart';

/// A custom linear progress bar for file transfers.
class SwiftWaveLinearProgress extends StatelessWidget {
  final double progress; // 0.0 to 1.0
  final bool isIndeterminate;

  const SwiftWaveLinearProgress({
    super.key,
    required this.progress,
    this.isIndeterminate = false,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: isIndeterminate
          ? LinearProgressIndicator(
              minHeight: 6,
              backgroundColor: theme.colorScheme.primary.withAlpha(20),
              valueColor: AlwaysStoppedAnimation<Color>(theme.colorScheme.primary),
            )
          : LinearProgressIndicator(
              value: progress,
              minHeight: 6,
              backgroundColor: theme.colorScheme.primary.withAlpha(20),
              valueColor: AlwaysStoppedAnimation<Color>(theme.colorScheme.primary),
            ),
    );
  }
}
