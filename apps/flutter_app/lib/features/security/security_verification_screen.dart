import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../design/components/swiftwave_buttons.dart';

class SecurityVerificationScreen extends StatelessWidget {
  const SecurityVerificationScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(title: const Text('Verify Security')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            const SizedBox(height: 24),
            Text(
              'Compare the code below with the code on the other device. If they match, your connection is secure.',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 48),
            Text(
              '1482',
              style: theme.textTheme.displayLarge?.copyWith(
                fontWeight: FontWeight.w700,
                letterSpacing: 8,
                color: theme.colorScheme.primary,
              ),
            ),
            const SizedBox(height: 48),
            Text(
              'Identity Fingerprint',
              style: theme.textTheme.titleSmall,
            ),
            const SizedBox(height: 8),
            Text(
              '3F:8A:22:9C:11:BB:CC',
              style: theme.textTheme.bodySmall?.copyWith(
                fontFamily: 'monospace',
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const Spacer(),
            SizedBox(
              width: double.infinity,
              child: SwiftWavePrimaryButton(
                label: 'Confirm Match',
                icon: Icons.verified_user_rounded,
                onPressed: () {
                  context.pop();
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Device marked as trusted.')),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
