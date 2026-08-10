/// Represents a discovered SwiftWave peer device.
///
/// This is a pure Dart model — it mirrors the `DeviceInfo` struct
/// from `swiftwave_core` and will be populated via the FFI bridge in Phase 2.
class Device {
  /// Stable device identifier (UUID string from swiftwave_core).
  final String id;

  /// Human-readable device name announced during discovery.
  final String displayName;

  /// Application version string of the remote device.
  final String appVersion;

  const Device({
    required this.id,
    required this.displayName,
    required this.appVersion,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) || (other is Device && other.id == id);

  @override
  int get hashCode => id.hashCode;

  @override
  String toString() => 'Device(id: $id, name: $displayName)';
}
