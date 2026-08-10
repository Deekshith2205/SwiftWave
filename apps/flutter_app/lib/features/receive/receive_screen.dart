import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../design/components/swiftwave_avatar.dart';
import '../../../design/components/swiftwave_buttons.dart';
import '../../../design/components/swiftwave_cards.dart';
import '../../../shared/providers/mock_providers.dart';

class ReceiveScreen extends ConsumerWidget {
  const ReceiveScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final settings = ref.watch(settingsProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Incoming Request')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              const SizedBox(height: 32),
              const SwiftWaveAvatar(deviceName: 'Aryan\'s Laptop', size: 80),
              const SizedBox(height: 24),
              Text(
                'Aryan\'s Laptop',
                style: theme.textTheme.headlineSmall?.copyWith(fontWeight: FontWeight.w700),
              ),
              const SizedBox(height: 8),
              Text(
                'wants to send you files',
                style: theme.textTheme.titleMedium?.copyWith(color: theme.colorScheme.onSurfaceVariant),
              ),
              
              const SizedBox(height: 48),
              SwiftWaveCard(
                child: Column(
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Text('Files'),
                        Text('12 items', style: TextStyle(color: theme.colorScheme.onSurfaceVariant)),
                      ],
                    ),
                    const Divider(height: 32),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Text('Total Size'),
                        Text('452 MB', style: TextStyle(color: theme.colorScheme.onSurfaceVariant)),
                      ],
                    ),
                    const Divider(height: 32),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Text('Destination'),
                        Text(settings.downloadPath, style: TextStyle(color: theme.colorScheme.onSurfaceVariant)),
                      ],
                    ),
                  ],
                ),
              ),

              const Spacer(),
              Row(
                children: [
                  Expanded(
                    child: SwiftWaveSecondaryButton(
                      label: 'Reject',
                      onPressed: () => context.pop(),
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: SwiftWavePrimaryButton(
                      label: 'Accept',
                      onPressed: () {
                        context.pop();
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(content: Text('Transfer started')),
                        );
                      },
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
