import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:swiftwave_app/core/ffi/swiftwave_native.dart';
import 'package:swiftwave_app/core/models/discovery.dart';
import 'package:swiftwave_app/core/providers/discovery_provider.dart';
import 'package:swiftwave_app/core/providers/runtime_provider.dart';

class MockSwiftWaveNative extends SwiftWaveNative {
  bool startCalled = false;
  bool stopCalled = false;

  final StreamController<DiscoveryEvent> _controller =
      StreamController<DiscoveryEvent>.broadcast();

  @override
  Stream<DiscoveryEvent> startDiscovery() {
    startCalled = true;
    return _controller.stream;
  }

  @override
  void stopDiscovery() {
    stopCalled = true;
  }

  void emit(DiscoveryEvent event) {
    _controller.add(event);
  }

  void complete() {
    _controller.close();
  }
}

class DegradedSwiftWaveNative extends SwiftWaveNative {
  @override
  Stream<DiscoveryEvent> startDiscovery() {
    throw Exception('Degraded mode');
  }

  @override
  void stopDiscovery() {}
}

void main() {
  test('PeerFound adds peer', () async {
    final mockNative = MockSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) => Future.value(mockNative))
      ],
    );
    addTearDown(container.dispose);

    container.listen(discoveryProvider, (_, _) {});
    await container.read(swiftWaveRuntimeProvider.future);

    expect(mockNative.startCalled, isTrue);

    final peer = const DiscoveredPeer(
      fingerprint: '123',
      displayName: 'Phone',
      address: '192.168.1.5',
      medium: DiscoveryMedium.mdnsUdp,
      protocolVersion: 1,
      lastSeen: 100,
    );

    mockNative.emit(DiscoveryEvent.peerFound(peer));
    await Future<void>.delayed(Duration.zero);

    final state = container.read(discoveryProvider);
    expect(state.length, 1);
    expect(state.first.fingerprint, '123');
  });

  test('duplicate PeerFound does not duplicate', () async {
    final mockNative = MockSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) => Future.value(mockNative))
      ],
    );
    addTearDown(container.dispose);

    container.listen(discoveryProvider, (_, _) {});
    await container.read(swiftWaveRuntimeProvider.future);

    final peer = const DiscoveredPeer(
      fingerprint: '123',
      displayName: 'Phone',
      address: '192.168.1.5',
      medium: DiscoveryMedium.mdnsUdp,
      protocolVersion: 1,
      lastSeen: 100,
    );

    mockNative.emit(DiscoveryEvent.peerFound(peer));
    await Future<void>.delayed(Duration.zero);
    mockNative.emit(DiscoveryEvent.peerFound(peer));
    await Future<void>.delayed(Duration.zero);

    final state = container.read(discoveryProvider);
    expect(state.length, 1);
  });

  test('PeerFound updates existing peer if appropriate', () async {
    final mockNative = MockSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) => Future.value(mockNative))
      ],
    );
    addTearDown(container.dispose);

    container.listen(discoveryProvider, (_, _) {});
    await container.read(swiftWaveRuntimeProvider.future);

    final peer1 = const DiscoveredPeer(
      fingerprint: '123',
      displayName: 'Phone',
      address: '192.168.1.5',
      medium: DiscoveryMedium.mdnsUdp,
      protocolVersion: 1,
      lastSeen: 100,
    );
    final peer2 = const DiscoveredPeer(
      fingerprint: '123',
      displayName: 'Phone Updated',
      address: '192.168.1.6',
      medium: DiscoveryMedium.mdnsUdp,
      protocolVersion: 1,
      lastSeen: 101,
    );

    mockNative.emit(DiscoveryEvent.peerFound(peer1));
    await Future<void>.delayed(Duration.zero);
    mockNative.emit(DiscoveryEvent.peerFound(peer2));
    await Future<void>.delayed(Duration.zero);

    final state = container.read(discoveryProvider);
    expect(state.length, 1);
    expect(state.first.displayName, 'Phone Updated');
    expect(state.first.address, '192.168.1.6');
  });

  test('PeerLost removes peer', () async {
    final mockNative = MockSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) => Future.value(mockNative))
      ],
    );
    addTearDown(container.dispose);

    container.listen(discoveryProvider, (_, _) {});
    await container.read(swiftWaveRuntimeProvider.future);

    final peer = const DiscoveredPeer(
      fingerprint: '123',
      displayName: 'Phone',
      address: '192.168.1.5',
      medium: DiscoveryMedium.mdnsUdp,
      protocolVersion: 1,
      lastSeen: 100,
    );

    mockNative.emit(DiscoveryEvent.peerFound(peer));
    await Future<void>.delayed(Duration.zero);
    expect(container.read(discoveryProvider).length, 1);

    mockNative.emit(DiscoveryEvent.peerLost('123'));
    await Future<void>.delayed(Duration.zero);
    expect(container.read(discoveryProvider).length, 0);
  });

  test('multiple peers coexist', () async {
    final mockNative = MockSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) => Future.value(mockNative))
      ],
    );
    addTearDown(container.dispose);

    container.listen(discoveryProvider, (_, _) {});
    await container.read(swiftWaveRuntimeProvider.future);

    final peer1 = const DiscoveredPeer(
      fingerprint: '1',
      displayName: 'Phone 1',
      address: '192.168.1.1',
      medium: DiscoveryMedium.mdnsUdp,
      protocolVersion: 1,
      lastSeen: 100,
    );
    final peer2 = const DiscoveredPeer(
      fingerprint: '2',
      displayName: 'Phone 2',
      address: '192.168.1.2',
      medium: DiscoveryMedium.mdnsUdp,
      protocolVersion: 1,
      lastSeen: 100,
    );

    mockNative.emit(DiscoveryEvent.peerFound(peer1));
    mockNative.emit(DiscoveryEvent.peerFound(peer2));
    await Future<void>.delayed(Duration.zero);

    expect(container.read(discoveryProvider).length, 2);
  });

  test('provider disposal stops/cleans up discovery', () async {
    final mockNative = MockSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) => Future.value(mockNative))
      ],
    );
    container.listen(discoveryProvider, (_, _) {});
    await container.read(swiftWaveRuntimeProvider.future);

    expect(mockNative.startCalled, isTrue);
    expect(mockNative.stopCalled, isFalse);

    container.dispose();
    expect(mockNative.stopCalled, isTrue);
  });

  test('degraded/native-unavailable behavior does not crash', () async {
    final degradedNative = DegradedSwiftWaveNative();
    final container = ProviderContainer(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) => Future.value(degradedNative))
      ],
    );
    addTearDown(container.dispose);

    // This should not crash, it should just fail silently as per the provider implementation
    container.listen(discoveryProvider, (_, _) {});
    await container.read(swiftWaveRuntimeProvider.future);
    
    expect(container.read(discoveryProvider), isEmpty);
  });
}
