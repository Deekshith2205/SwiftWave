import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:swiftwave_app/core/models/authenticated_peer.dart';
import 'package:swiftwave_app/core/providers/authenticated_peer_provider.dart';
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

void main() {
  testWidgets('HomeScreen displays authenticated peers', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          authenticatedPeerProvider.overrideWith((ref) {
            return _FakeNotifier();
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
  });
}
