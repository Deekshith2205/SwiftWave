import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../shared/providers/mock_providers.dart';
import '../../app/router.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);
    final notifier = ref.read(settingsProvider.notifier);

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        children: [
          _buildSectionHeader(context, 'General'),
          ListTile(
            title: const Text('Device Name'),
            subtitle: Text(settings.deviceName),
            onTap: () {},
          ),
          ListTile(
            title: const Text('Download Location'),
            subtitle: Text(settings.downloadPath),
            onTap: () {},
          ),
          
          _buildSectionHeader(context, 'Transfer'),
          SwitchListTile(
            title: const Text('Auto-accept from trusted'),
            subtitle: const Text('Skip prompt for verified devices'),
            value: settings.autoAcceptTrusted,
            onChanged: (val) => notifier.updateSettings(settings.copyWith(autoAcceptTrusted: val)),
          ),
          SwitchListTile(
            title: const Text('Compress before sending'),
            value: settings.enableCompression,
            onChanged: (val) => notifier.updateSettings(settings.copyWith(enableCompression: val)),
          ),
          
          _buildSectionHeader(context, 'Appearance'),
          ListTile(
            title: const Text('Theme'),
            subtitle: Text(settings.themeMode),
            onTap: () {},
          ),
          
          _buildSectionHeader(context, 'About'),
          ListTile(
            title: const Text('About SwiftWave'),
            onTap: () => context.push(AppRoutes.about),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(24, 24, 24, 8),
      child: Text(
        title.toUpperCase(),
        style: theme.textTheme.labelSmall?.copyWith(
          color: theme.colorScheme.primary,
          letterSpacing: 1.2,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}
