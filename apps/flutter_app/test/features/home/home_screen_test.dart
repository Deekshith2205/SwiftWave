import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:swiftwave_app/core/models/authenticated_peer.dart';
import 'package:swiftwave_app/core/providers/authenticated_peer_provider.dart';
import 'package:swiftwave_app/core/models/discovery.dart';
import 'package:swiftwave_app/core/providers/discovery_provider.dart';
import 'package:swiftwave_app/features/home/home_screen.dart';

class _FakeNotifier extends AuthenticatedPeerNotifier {
  _FakeNotifier() {
    state = [
      const AuthenticatedPeer(
        fingerprint: '1234567890abcdef',
        displayName: 'My Trusted PC',
      ),
    ];
  }
}

class _FakeDiscoveryNotifier extends DiscoveryNotifier {
  _FakeDiscoveryNotifier() {
    state = [
      const DiscoveredPeer(
        fingerprint: 'abcdabcdabcd',
        displayName: 'A Discovered Phone',
        address: '192.168.1.100',
        medium: DiscoveryMedium.mdnsUdp,
        protocolVersion: 1,
        lastSeen: 10000,
      ),
    ];
  }
}

void main() {
  testWidgets('HomeScreen displays authenticated peers and discovered peers', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          authenticatedPeerProvider.overrideWith((ref) {
            return _FakeNotifier();
          }),
          discoveryProvider.overrideWith((ref) {
            return _FakeDiscoveryNotifier();
          }),
        ],
        child: const MaterialApp(home: HomeScreen()),
      ),
    );

    await tester.pumpAndSettle();

    // Verify UI semantic elements
    expect(find.text('Authenticated Devices'), findsOneWidget);
    expect(find.text('My Trusted PC'), findsOneWidget);
    expect(find.textContaining('Authenticated • 12345678'), findsOneWidget);
    
    // Verify Discovered Peers
    expect(find.text('Nearby Devices'), findsOneWidget);
    expect(find.text('A Discovered Phone'), findsOneWidget);
  });
}
