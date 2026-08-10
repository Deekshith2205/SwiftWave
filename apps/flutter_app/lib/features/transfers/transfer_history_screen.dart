import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/models/transfer.dart';
import '../../../design/components/swiftwave_cards.dart';
import '../../../shared/providers/mock_providers.dart';

class TransferHistoryScreen extends ConsumerWidget {
  const TransferHistoryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final history = ref.watch(transferHistoryProvider);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(title: const Text('Transfer History')),
      body: ListView.separated(
        padding: const EdgeInsets.all(24),
        itemCount: history.length,
        separatorBuilder: (context, index) => const SizedBox(height: 12),
        itemBuilder: (context, index) {
          final transfer = history[index];
          final sizeMB = (transfer.fileSize / 1000000).toStringAsFixed(1);
          final isSuccess = transfer.status == TransferStatus.completed;

          return SwiftWaveCard(
            child: Row(
              children: [
                Icon(
                  transfer.isSender ? Icons.upload_rounded : Icons.download_rounded,
                  color: theme.colorScheme.primary,
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(transfer.fileName, style: theme.textTheme.titleSmall),
                      Text('$sizeMB MB • ${DateTime.now().toString().split(' ')[0]}',
                          style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
                    ],
                  ),
                ),
                if (isSuccess)
                  const Icon(Icons.check_circle_rounded, color: Colors.green, size: 20)
                else
                  const Icon(Icons.error_outline_rounded, color: Colors.red, size: 20),
              ],
            ),
          );
        },
      ),
    );
  }
}
