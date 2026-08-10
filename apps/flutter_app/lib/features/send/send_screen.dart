import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../design/components/swiftwave_buttons.dart';
import '../../../design/components/swiftwave_cards.dart';

class SendScreen extends ConsumerStatefulWidget {
  const SendScreen({super.key});

  @override
  ConsumerState<SendScreen> createState() => _SendScreenState();
}

class _SendScreenState extends ConsumerState<SendScreen> {
  bool _compress = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(title: const Text('Send Files')),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(24),
          children: [
            Text(
              'Select Files',
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            SwiftWaveCard(
              onTap: () {
                // TODO: Open file picker
              },
              child: const Padding(
                padding: EdgeInsets.symmetric(vertical: 32),
                child: Center(
                  child: Column(
                    children: [
                      Icon(Icons.add_circle_outline_rounded, size: 48),
                      SizedBox(height: 12),
                      Text('Tap to select files or folders'),
                    ],
                  ),
                ),
              ),
            ),
            
            const SizedBox(height: 32),
            Text(
              'Selected (0)',
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              'Total size: 0 MB\nEstimated transfer: 0 MB',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            
            const SizedBox(height: 32),
            SwitchListTile(
              title: const Text('Enable Compression'),
              subtitle: const Text('Reduces transfer size but uses CPU'),
              value: _compress,
              onChanged: (val) => setState(() => _compress = val),
              contentPadding: EdgeInsets.zero,
            ),
            
            const SizedBox(height: 48),
            SwiftWavePrimaryButton(
              label: 'Send',
              icon: Icons.send_rounded,
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('Please select files first.')),
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}
