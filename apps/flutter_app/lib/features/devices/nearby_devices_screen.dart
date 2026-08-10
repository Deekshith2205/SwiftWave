import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../design/components/swiftwave_avatar.dart';
import '../../../design/components/swiftwave_cards.dart';
import '../../../shared/providers/mock_providers.dart';
import '../../app/router.dart';

class NearbyDevicesScreen extends ConsumerWidget {
  const NearbyDevicesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final devices = ref.watch(mockDevicesProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Nearby Devices')),
      body: ListView.separated(
        padding: const EdgeInsets.all(24),
        itemCount: devices.length,
        separatorBuilder: (context, index) => const SizedBox(height: 12),
        itemBuilder: (context, index) {
          final device = devices[index];
          return SwiftWaveCard(
            onTap: () => context.push('${AppRoutes.devices}/${device.id}'),
            child: Row(
              children: [
                SwiftWaveAvatar(deviceName: device.displayName),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(device.displayName, style: Theme.of(context).textTheme.titleSmall),
                      Text('Nearby', style: Theme.of(context).textTheme.bodySmall),
                    ],
                  ),
                ),
                const Icon(Icons.chevron_right_rounded),
              ],
            ),
          );
        },
      ),
    );
  }
}
