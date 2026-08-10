import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/models/transfer.dart';
import '../../../design/components/swiftwave_buttons.dart';
import '../../../design/components/swiftwave_progress.dart';
import '../../../shared/providers/mock_providers.dart';

class TransferDetailsScreen extends ConsumerWidget {
  final String transferId;

  const TransferDetailsScreen({super.key, required this.transferId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final activeTransfers = ref.watch(activeTransfersProvider);
    final transfer = activeTransfers.firstWhere(
      (t) => t.id == transferId,
      orElse: () => activeTransfers.first,
    );

    final totalSizeMB = (transfer.fileSize / 1000000).toStringAsFixed(1);
    final currentSizeMB = (transfer.bytesTransferred / 1000000).toStringAsFixed(1);
    final isDone = transfer.status == TransferStatus.completed;

    return Scaffold(
      appBar: AppBar(title: const Text('Transfer Details')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              transfer.fileName,
              style: theme.textTheme.headlineSmall?.copyWith(fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 8),
            Text(
              isDone ? 'Transfer complete' : 'Transferring...',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: isDone ? Colors.green : theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 48),
            Row(
              children: [
                Expanded(
                  child: SwiftWaveLinearProgress(progress: transfer.progress),
                ),
                const SizedBox(width: 16),
                Text(
                  '${(transfer.progress * 100).toInt()}%',
                  style: theme.textTheme.titleMedium?.copyWith(
                    color: theme.colorScheme.primary,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 32),
            _buildInfoRow(context, 'Progress', '$currentSizeMB MB / $totalSizeMB MB'),
            const Divider(height: 32),
            _buildInfoRow(context, 'Speed', isDone ? '-' : '18.4 MB/s'),
            const Divider(height: 32),
            _buildInfoRow(context, 'ETA', isDone ? '-' : '1 sec'),
            const Divider(height: 32),
            _buildInfoRow(context, 'Connection', 'Direct Wi-Fi'),
            const Spacer(),
            if (!isDone)
              SizedBox(
                width: double.infinity,
                child: SwiftWaveSecondaryButton(
                  label: 'Cancel Transfer',
                  icon: Icons.cancel_rounded,
                  onPressed: () {},
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoRow(BuildContext context, String label, String value) {
    final theme = Theme.of(context);
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: theme.textTheme.bodyMedium?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        Text(
          value,
          style: theme.textTheme.bodyMedium?.copyWith(
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    );
  }
}
