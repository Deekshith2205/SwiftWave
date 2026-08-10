import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/models/device.dart';
import '../../../core/models/transfer.dart';
import '../../../design/components/swiftwave_avatar.dart';
import '../../../design/components/swiftwave_buttons.dart';
import '../../../design/components/swiftwave_cards.dart';
import '../../../design/components/swiftwave_progress.dart';
import '../../../design/components/swiftwave_status.dart';
import '../../../shared/providers/mock_providers.dart';
import '../../app/router.dart';

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final settings = ref.watch(settingsProvider);
    final nearbyDevices = ref.watch(mockDevicesProvider);
    final activeTransfers = ref.watch(activeTransfersProvider);

    return Scaffold(
      body: SafeArea(
        child: CustomScrollView(
          slivers: [
            // App header
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(24, 32, 24, 24),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'SWIFTWAVE',
                      style: theme.textTheme.labelMedium?.copyWith(
                        letterSpacing: 2,
                        color: theme.colorScheme.primary,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const SizedBox(height: 12),
                    Text(
                      settings.deviceName,
                      style: theme.textTheme.headlineMedium?.copyWith(
                        fontWeight: FontWeight.w700,
                        color: theme.colorScheme.onSurface,
                      ),
                    ),
                    const SizedBox(height: 8),
                    const SwiftWaveStatusBadge(
                      type: SwiftWaveStatusType.online,
                      label: 'Ready to share',
                    ),
                    const SizedBox(height: 32),
                    Row(
                      children: [
                        Expanded(
                          child: SwiftWavePrimaryButton(
                            label: 'SEND FILES',
                            icon: Icons.upload_rounded,
                            onPressed: () => context.push(AppRoutes.send),
                          ),
                        ),
                        const SizedBox(width: 16),
                        Expanded(
                          child: SwiftWaveSecondaryButton(
                            label: 'RECEIVE',
                            icon: Icons.download_rounded,
                            onPressed: () => context.push(AppRoutes.receive),
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),

            // Nearby Devices
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(
                      'Nearby Devices',
                      style: theme.textTheme.titleMedium,
                    ),
                    SwiftWaveGhostButton(
                      label: 'View All',
                      onPressed: () => context.push(AppRoutes.devices),
                    ),
                  ],
                ),
              ),
            ),
            if (nearbyDevices.isEmpty)
              const SliverToBoxAdapter(
                child: Padding(
                  padding: EdgeInsets.symmetric(horizontal: 24, vertical: 8),
                  child: Text('Scanning for nearby devices...'),
                ),
              )
            else
              SliverPadding(
                padding: const EdgeInsets.symmetric(horizontal: 24),
                sliver: SliverList(
                  delegate: SliverChildBuilderDelegate(
                    (context, index) {
                      final device = nearbyDevices[index];
                      return Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: _DeviceCard(device: device),
                      );
                    },
                    childCount: nearbyDevices.length > 3 ? 3 : nearbyDevices.length,
                  ),
                ),
              ),

            // Active Transfers
            const SliverToBoxAdapter(child: SizedBox(height: 16)),
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(
                      'Transfer Activity',
                      style: theme.textTheme.titleMedium,
                    ),
                    if (activeTransfers.isNotEmpty)
                      SwiftWaveGhostButton(
                        label: 'History',
                        onPressed: () => context.push(AppRoutes.history),
                      ),
                  ],
                ),
              ),
            ),
            if (activeTransfers.isEmpty)
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
                  child: Text(
                    'No active transfers',
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
              )
            else
              SliverPadding(
                padding: const EdgeInsets.symmetric(horizontal: 24),
                sliver: SliverList(
                  delegate: SliverChildBuilderDelegate(
                    (context, index) {
                      final transfer = activeTransfers[index];
                      return Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: _ActiveTransferCard(transfer: transfer),
                      );
                    },
                    childCount: activeTransfers.length,
                  ),
                ),
              ),
              
            const SliverToBoxAdapter(child: SizedBox(height: 48)),
          ],
        ),
      ),
    );
  }
}

class _DeviceCard extends StatelessWidget {
  final Device device;

  const _DeviceCard({required this.device});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
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
                Text(
                  device.displayName,
                  style: theme.textTheme.titleSmall,
                ),
                Text(
                  'Nearby • Ready',
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
          SwiftWaveGhostButton(
            label: 'Connect',
            onPressed: () => context.push(AppRoutes.send),
          ),
        ],
      ),
    );
  }
}

class _ActiveTransferCard extends StatelessWidget {
  final Transfer transfer;

  const _ActiveTransferCard({required this.transfer});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final progress = transfer.progress;
    
    // Formatting sizes (Mock)
    final totalSizeMB = (transfer.fileSize / 1000000).toStringAsFixed(1);
    final currentSizeMB = (transfer.bytesTransferred / 1000000).toStringAsFixed(1);

    return SwiftWaveCard(
      onTap: () => context.push('${AppRoutes.transfers}/${transfer.id}'),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(
                child: Text(
                  transfer.fileName,
                  style: theme.textTheme.titleSmall,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (transfer.status == TransferStatus.completed)
                const Icon(Icons.check_circle_rounded, color: Colors.green, size: 20),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: SwiftWaveLinearProgress(progress: progress),
              ),
              const SizedBox(width: 12),
              Text(
                '${(progress * 100).toInt()}%',
                style: theme.textTheme.labelMedium?.copyWith(
                  fontWeight: FontWeight.w700,
                  color: theme.colorScheme.primary,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                '$currentSizeMB MB / $totalSizeMB MB',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
              if (transfer.status == TransferStatus.active)
                Text(
                  '18.4 MB/s', // Mock speed
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
            ],
          ),
          if (transfer.status == TransferStatus.active) ...[
            const SizedBox(height: 4),
            Text(
              'ETA 1 sec', // Mock ETA
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
          ]
        ],
      ),
    );
  }
}
