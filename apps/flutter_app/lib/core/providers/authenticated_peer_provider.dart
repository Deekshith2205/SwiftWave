import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../ffi/swiftwave_native.dart';
import '../models/authenticated_peer.dart';
import 'runtime_provider.dart';

class AuthenticatedPeerNotifier extends StateNotifier<List<AuthenticatedPeer>> {
  SwiftWaveNative? _native;
  StreamSubscription<AuthenticatedPeerEvent>? _subscription;

  AuthenticatedPeerNotifier() : super([]);

  void attach(SwiftWaveNative native) {
    if (_native == native) return;

    _subscription?.cancel();
    _native?.stopAuthenticatedPeers();

    _native = native;
    try {
      _subscription = _native!.subscribeAuthenticatedPeers().listen((event) {
        if (event is IncomingAuthenticatedPeer) {
          // Check for duplicates before adding
          if (!state.any((p) => p.fingerprint == event.peer.fingerprint)) {
            state = [...state, event.peer];
          }
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
