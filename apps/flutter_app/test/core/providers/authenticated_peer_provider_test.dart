import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:swiftwave_app/core/providers/authenticated_peer_provider.dart';
import 'package:swiftwave_app/core/providers/runtime_provider.dart';
import 'package:swiftwave_app/core/ffi/swiftwave_native.dart';
import 'package:swiftwave_app/core/models/authenticated_peer.dart';
import 'package:swiftwave_app/core/models/discovery.dart';

class MockSwiftWaveNative implements SwiftWaveNative {
  bool stopCalled = false;
  final StreamController<AuthenticatedPeer> _controller =
      StreamController.broadcast();

  @override
  void create({required String dataDirectory}) {}

  @override
  void init() {}

  @override
  void shutdown() {}

  @override
  void destroy() {}

  @override
  String getDeviceId() => 'test-device-id';

  @override
  String getVersion() => '1.0.0';

  @override
  Stream<DiscoveryEvent> startDiscovery() => const Stream.empty();

  @override
  void stopDiscovery() {}

  @override
  Stream<AuthenticatedPeer> subscribeAuthenticatedPeers() => _controller.stream;

  @override
  void stopAuthenticatedPeers() {
    stopCalled = true;
  }

  void simulatePeer(AuthenticatedPeer peer) {
    _controller.add(peer);
  }
}

void main() {
  test('AuthenticatedPeerNotifier initial state is empty', () {
    final notifier = AuthenticatedPeerNotifier();
    expect(notifier.state, isEmpty);
  });

  test('Authenticated peer arrives updates state', () async {
    final mockNative = MockSwiftWaveNative();
    final notifier = AuthenticatedPeerNotifier();
    notifier.attach(mockNative);

    mockNative.simulatePeer(
      const AuthenticatedPeer(fingerprint: 'abc', displayName: 'Test Device'),
    );

    await Future<void>.delayed(Duration.zero);
    expect(notifier.state.length, 1);
    expect(notifier.state.first.fingerprint, 'abc');
    expect(notifier.state.first.displayName, 'Test Device');
  });

  test('Duplicate event with same fingerprint ignores second', () async {
    final mockNative = MockSwiftWaveNative();
    final notifier = AuthenticatedPeerNotifier();
    notifier.attach(mockNative);

    mockNative.simulatePeer(
      const AuthenticatedPeer(fingerprint: 'abc', displayName: 'Device 1'),
    );
    mockNative.simulatePeer(
      const AuthenticatedPeer(fingerprint: 'abc', displayName: 'Device 1'),
    );

    await Future<void>.delayed(Duration.zero);
    expect(notifier.state.length, 1);
  });

  test('Different fingerprints adds multiple peers', () async {
    final mockNative = MockSwiftWaveNative();
    final notifier = AuthenticatedPeerNotifier();
    notifier.attach(mockNative);

    mockNative.simulatePeer(
      const AuthenticatedPeer(fingerprint: 'A', displayName: 'Device A'),
    );
    mockNative.simulatePeer(
      const AuthenticatedPeer(fingerprint: 'B', displayName: 'Device B'),
    );

    await Future<void>.delayed(Duration.zero);
    expect(notifier.state.length, 2);
  });

  test('Same fingerprint, different display name keeps one peer', () async {
    final mockNative = MockSwiftWaveNative();
    final notifier = AuthenticatedPeerNotifier();
    notifier.attach(mockNative);

    mockNative.simulatePeer(
      const AuthenticatedPeer(fingerprint: 'A', displayName: 'Device 1'),
    );
    mockNative.simulatePeer(
      const AuthenticatedPeer(fingerprint: 'A', displayName: 'Device 2'),
    );

    await Future<void>.delayed(Duration.zero);
    expect(notifier.state.length, 1);
    expect(
      notifier.state.first.displayName,
      'Device 1',
    ); // state isn't overwritten
  });

  test('Provider disposal calls stopAuthenticatedPeers', () async {
    final mockNative = MockSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) async => mockNative),
      ],
    );

    final sub = container.listen(authenticatedPeerProvider, (_, _) {});

    await container.read(swiftWaveRuntimeProvider.future);

    // allow listener to run
    expect(mockNative.stopCalled, isFalse);

    sub.close();
    container.dispose();

    expect(mockNative.stopCalled, isTrue);
  });

  test('Runtime degraded mode does not crash', () {
    final notifier = AuthenticatedPeerNotifier();
    // In degraded mode attach could throw if native wasn't mocked properly,
    // but the test checks it handles errors safely.
    expect(() => notifier.attach(SwiftWaveNative()), returnsNormally);
  });
}
