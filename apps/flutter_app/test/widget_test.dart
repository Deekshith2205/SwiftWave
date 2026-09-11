import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:swiftwave_app/core/providers/runtime_provider.dart';
import 'package:swiftwave_app/core/ffi/swiftwave_native.dart';
import 'package:swiftwave_app/app/app.dart';
import 'package:swiftwave_app/shared/widgets/scaffold_shell.dart';

class FakeSwiftWaveNative implements SwiftWaveNative {
  @override
  void create({required String dataDirectory}) {}
  
  @override
  void init() {}
  
  @override
  void shutdown() {}
  
  @override
  void destroy() {}
  
  @override
  String getDeviceId() => '00000000-0000-0000-0000-000000000000';
  
  @override
  String getVersion() => '0.0.0-fake';
}

void main() {
  testWidgets('SwiftWave app launches and shows shell', (WidgetTester tester) async {
    // Build our app and trigger a frame.
    await tester.pumpWidget(ProviderScope(
      overrides: [
        swiftWaveRuntimeProvider.overrideWith((ref) async {
          return FakeSwiftWaveNative();
        }),
      ],
      child: const SwiftWaveApp(),
    ));
    await tester.pumpAndSettle();

    // Verify that the main app shell is rendered.
    expect(find.byType(ScaffoldShell), findsOneWidget);
  });
}
