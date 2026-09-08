import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:swiftwave_app/app/app.dart';
import 'package:swiftwave_app/shared/widgets/scaffold_shell.dart';

void main() {
  testWidgets('SwiftWave app launches and shows shell', (WidgetTester tester) async {
    // Build our app and trigger a frame.
    await tester.pumpWidget(const ProviderScope(child: SwiftWaveApp()));
    await tester.pumpAndSettle();

    // Verify that the main app shell is rendered.
    expect(find.byType(ScaffoldShell), findsOneWidget);
  });
}
