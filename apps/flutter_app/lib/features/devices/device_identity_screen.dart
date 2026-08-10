import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../design/components/swiftwave_avatar.dart';
import '../../../design/components/swiftwave_buttons.dart';
import '../../../shared/providers/mock_providers.dart';
import '../../app/router.dart';

class DeviceIdentityScreen extends ConsumerWidget {
  final String deviceId;

  const DeviceIdentityScreen({super.key, required this.deviceId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final devices = ref.watch(mockDevicesProvider);
    final device = devices.firstWhere(
      (d) => d.id == deviceId,
      orElse: () => devices.first, // fallback for mock
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Device Details')),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            const SizedBox(height: 32),
            SwiftWaveAvatar(deviceName: device.displayName, size: 80, isTrusted: true),
            const SizedBox(height: 24),
            Text(
              device.displayName,
              style: theme.textTheme.headlineMedium?.copyWith(fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 8),
            Text(
              'Trusted Device',
              style: theme.textTheme.titleMedium?.copyWith(color: Colors.green),
            ),
            const SizedBox(height: 48),
            ListTile(
              title: const Text('Device ID'),
              subtitle: Text(device.id),
              trailing: IconButton(
                icon: const Icon(Icons.copy_rounded),
                onPressed: () {},
              ),
              contentPadding: EdgeInsets.zero,
            ),
            const Divider(),
            ListTile(
              title: const Text('App Version'),
              subtitle: Text(device.appVersion),
              contentPadding: EdgeInsets.zero,
            ),
            const Divider(),
            const Spacer(),
            SizedBox(
              width: double.infinity,
              child: SwiftWavePrimaryButton(
                label: 'Send Files',
                onPressed: () => context.push(AppRoutes.send),
              ),
            ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: SwiftWaveSecondaryButton(
                label: 'Verify Security (SAS)',
                onPressed: () => context.push(AppRoutes.security),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
