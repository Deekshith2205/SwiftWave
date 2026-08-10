import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../features/home/home_screen.dart';
import '../features/send/send_screen.dart';
import '../features/receive/receive_screen.dart';
import '../features/devices/nearby_devices_screen.dart';
import '../features/devices/device_identity_screen.dart';
import '../features/transfers/transfer_details_screen.dart';
import '../features/transfers/transfer_history_screen.dart';
import '../features/security/security_verification_screen.dart';
import '../features/settings/settings_screen.dart';
import '../features/settings/about_screen.dart';
import '../shared/widgets/scaffold_shell.dart';

/// Centralised route names.
abstract class AppRoutes {
  static const home = '/';
  static const send = '/send';
  static const receive = '/receive';
  static const devices = '/devices';
  static const transfers = '/transfers';
  static const history = '/history';
  static const security = '/security';
  static const settings = '/settings';
  static const about = '/settings/about';
}

/// Provides the GoRouter instance for the app.
final routerProvider = Provider<GoRouter>((ref) {
  final rootNavigatorKey = GlobalKey<NavigatorState>();
  final shellNavigatorKey = GlobalKey<NavigatorState>();

  return GoRouter(
    navigatorKey: rootNavigatorKey,
    initialLocation: AppRoutes.home,
    routes: [
      // ----------------------------------------------------------------------
      // Shell Route (Screens with bottom navigation / sidebar)
      // ----------------------------------------------------------------------
      ShellRoute(
        navigatorKey: shellNavigatorKey,
        builder: (context, state, child) {
          return ScaffoldShell(child: child);
        },
        routes: [
          GoRoute(
            path: AppRoutes.home,
            builder: (context, state) => const HomeScreen(),
          ),
          GoRoute(
            path: AppRoutes.devices,
            builder: (context, state) => const NearbyDevicesScreen(),
          ),
          GoRoute(
            path: AppRoutes.history,
            builder: (context, state) => const TransferHistoryScreen(),
          ),
          GoRoute(
            path: AppRoutes.settings,
            builder: (context, state) => const SettingsScreen(),
          ),
        ],
      ),

      // ----------------------------------------------------------------------
      // Full-screen Routes (No bottom navigation)
      // ----------------------------------------------------------------------
      GoRoute(
        path: AppRoutes.send,
        parentNavigatorKey: rootNavigatorKey,
        builder: (context, state) => const SendScreen(),
      ),
      GoRoute(
        path: AppRoutes.receive,
        parentNavigatorKey: rootNavigatorKey,
        builder: (context, state) => const ReceiveScreen(),
      ),
      GoRoute(
        path: '${AppRoutes.devices}/:id',
        parentNavigatorKey: rootNavigatorKey,
        builder: (context, state) {
          final id = state.pathParameters['id']!;
          return DeviceIdentityScreen(deviceId: id);
        },
      ),
      GoRoute(
        path: '${AppRoutes.transfers}/:id',
        parentNavigatorKey: rootNavigatorKey,
        builder: (context, state) {
          final id = state.pathParameters['id']!;
          return TransferDetailsScreen(transferId: id);
        },
      ),
      GoRoute(
        path: AppRoutes.security,
        parentNavigatorKey: rootNavigatorKey,
        builder: (context, state) => const SecurityVerificationScreen(),
      ),
      GoRoute(
        path: AppRoutes.about,
        parentNavigatorKey: rootNavigatorKey,
        builder: (context, state) => const AboutScreen(),
      ),
    ],
  );
});
