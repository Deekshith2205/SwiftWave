import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/models/device.dart';
import '../../core/models/transfer.dart';
import '../../core/models/settings.dart';

// ---------------------------------------------------------------------------
// Mock Devices
// ---------------------------------------------------------------------------

final mockDevicesProvider = Provider<List<Device>>((ref) {
  return [
    const Device(
      id: 'dev-1',
      displayName: 'Aryan\'s Laptop',
      appVersion: '1.0.0',
    ),
    const Device(
      id: 'dev-2',
      displayName: 'Sarah\'s Phone',
      appVersion: '1.0.0',
    ),
  ];
});

// ---------------------------------------------------------------------------
// Mock Active Transfers (Simulates progress)
// ---------------------------------------------------------------------------

class MockActiveTransfersNotifier extends StateNotifier<List<Transfer>> {
  MockActiveTransfersNotifier() : super([
    const Transfer(
      id: 'tx-1',
      fileName: 'IMG_2048.jpg',
      fileSize: 58000000,
      bytesTransferred: 0,
      isSender: true,
      remotePeerId: 'dev-1',
      status: TransferStatus.active,
    )
  ]) {
    _startSimulating();
  }

  Timer? _timer;

  void _startSimulating() {
    _timer = Timer.periodic(const Duration(milliseconds: 500), (timer) {
      if (!mounted) return;
      state = state.map((tx) {
        if (tx.status != TransferStatus.active) return tx;

        final newBytes = tx.bytesTransferred + (18400000 ~/ 2); // Simulating ~18.4 MB/s
        if (newBytes >= tx.fileSize) {
          return Transfer(
            id: tx.id,
            fileName: tx.fileName,
            fileSize: tx.fileSize,
            bytesTransferred: tx.fileSize,
            isSender: tx.isSender,
            remotePeerId: tx.remotePeerId,
            status: TransferStatus.completed,
          );
        }
        return Transfer(
          id: tx.id,
          fileName: tx.fileName,
          fileSize: tx.fileSize,
          bytesTransferred: newBytes,
          isSender: tx.isSender,
          remotePeerId: tx.remotePeerId,
          status: tx.status,
        );
      }).toList();
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }
}

final activeTransfersProvider =
    StateNotifierProvider<MockActiveTransfersNotifier, List<Transfer>>((ref) {
  return MockActiveTransfersNotifier();
});

// ---------------------------------------------------------------------------
// Mock Transfer History
// ---------------------------------------------------------------------------

final transferHistoryProvider = Provider<List<Transfer>>((ref) {
  return [
    const Transfer(
      id: 'tx-old-1',
      fileName: 'Project_Backup.zip',
      fileSize: 1200000000,
      bytesTransferred: 1200000000,
      isSender: false,
      remotePeerId: 'dev-2',
      status: TransferStatus.completed,
    ),
    const Transfer(
      id: 'tx-old-2',
      fileName: 'video_draft.mp4',
      fileSize: 450000000,
      bytesTransferred: 120000000,
      isSender: true,
      remotePeerId: 'dev-1',
      status: TransferStatus.cancelled,
    ),
  ];
});

// ---------------------------------------------------------------------------
// Mock Settings
// ---------------------------------------------------------------------------

class MockSettingsNotifier extends StateNotifier<AppSettings> {
  MockSettingsNotifier()
      : super(const AppSettings(
          deviceName: 'My SwiftWave Device',
          downloadPath: '/Downloads/SwiftWave',
          autoAcceptTrusted: true,
          enableCompression: false,
          themeMode: 'system',
          discoveryTimeoutSeconds: 120,
        ));

  void updateSettings(AppSettings newSettings) {
    state = newSettings;
  }
}

final settingsProvider =
    StateNotifierProvider<MockSettingsNotifier, AppSettings>((ref) {
  return MockSettingsNotifier();
});
