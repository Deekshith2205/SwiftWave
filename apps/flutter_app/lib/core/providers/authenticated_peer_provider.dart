import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../ffi/swiftwave_native.dart';
import '../models/authenticated_peer.dart';
import 'runtime_provider.dart';

class AuthenticatedPeerNotifier extends StateNotifier<List<AuthenticatedPeer>> {
  SwiftWaveNative? _native;
  StreamSubscription<AuthenticatedPeer>? _subscription;

  AuthenticatedPeerNotifier() : super([]);

  void attach(SwiftWaveNative native) {
    if (_native == native) return;

    _subscription?.cancel();
    _native?.stopAuthenticatedPeers();

    _native = native;
    try {
      _subscription = _native!.subscribeAuthenticatedPeers().listen((peer) {
        // Check for duplicates before adding, using fingerprint as identity key
        if (!state.any((p) => p.fingerprint == peer.fingerprint)) {
          state = [...state, peer];
        } else {
          // If the fingerprint exists but the display name changed, we could update it,
          // but the instructions say "E. Same fingerprint, different display name ... Expected: still one peer."
          // So we don't modify the state.
        }
      });
    } catch (e) {
      // Degraded mode / failed to start
    }
  }

  @override
  void dispose() {
    _subscription?.cancel();
    _native?.stopAuthenticatedPeers();
    super.dispose();
  }
}

final authenticatedPeerProvider =
    StateNotifierProvider<AuthenticatedPeerNotifier, List<AuthenticatedPeer>>((
      ref,
    ) {
      final notifier = AuthenticatedPeerNotifier();

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
