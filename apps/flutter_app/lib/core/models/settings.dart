/// Settings model for the app.
class AppSettings {
  final String deviceName;
  final String downloadPath;
  final bool autoAcceptTrusted;
  final bool enableCompression;
  final String themeMode; // 'system', 'light', 'dark'
  final int discoveryTimeoutSeconds;

  const AppSettings({
    required this.deviceName,
    required this.downloadPath,
    required this.autoAcceptTrusted,
    required this.enableCompression,
    required this.themeMode,
    required this.discoveryTimeoutSeconds,
  });

  AppSettings copyWith({
    String? deviceName,
    String? downloadPath,
    bool? autoAcceptTrusted,
    bool? enableCompression,
    String? themeMode,
    int? discoveryTimeoutSeconds,
  }) {
    return AppSettings(
      deviceName: deviceName ?? this.deviceName,
      downloadPath: downloadPath ?? this.downloadPath,
      autoAcceptTrusted: autoAcceptTrusted ?? this.autoAcceptTrusted,
      enableCompression: enableCompression ?? this.enableCompression,
      themeMode: themeMode ?? this.themeMode,
      discoveryTimeoutSeconds:
          discoveryTimeoutSeconds ?? this.discoveryTimeoutSeconds,
    );
  }
}
