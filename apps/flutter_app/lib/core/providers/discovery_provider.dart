import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../ffi/swiftwave_native.dart';
import '../models/discovery.dart';
import 'runtime_provider.dart';

class DiscoveryNotifier extends StateNotifier<List<DiscoveredPeer>> {
  SwiftWaveNative? _native;
  StreamSubscription<DiscoveryEvent>? _subscription;

  DiscoveryNotifier() : super([]);

  void attach(SwiftWaveNative native) {
    if (_native == native) return;

    _subscription?.cancel();
    _native?.stopDiscovery();

    _native = native;
    try {
      _subscription = _native!.startDiscovery().listen((event) {
        if (event is PeerFound) {
          final peer = event.peer;
          final existingIndex =
              state.indexWhere((p) => p.fingerprint == peer.fingerprint);
          if (existingIndex >= 0) {
            // Update existing peer
            final newState = [...state];
            newState[existingIndex] = peer;
            state = newState;
          } else {
            // Add new peer
            state = [...state, peer];
          }
        } else if (event is PeerLost) {
          final fingerprint = event.fingerprint;
          state =
              state.where((peer) => peer.fingerprint != fingerprint).toList();
        }
      });
    } catch (e) {
      // Degraded mode / failed to start
    }
  }

  @override
  void dispose() {
    _subscription?.cancel();
    _native?.stopDiscovery();
    super.dispose();
  }
}

final discoveryProvider =
    StateNotifierProvider<DiscoveryNotifier, List<DiscoveredPeer>>((ref) {
  final notifier = DiscoveryNotifier();

  ref.listen<AsyncValue<SwiftWaveNative>>(swiftWaveRuntimeProvider, (
    previous,
    next,
  ) {
    if (next.hasValue && next.value != null) {
      notifier.attach(next.value!);
    }
  }, fireImmediately: true);

  return notifier;
});
