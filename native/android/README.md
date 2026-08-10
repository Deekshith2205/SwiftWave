# Android Native Adapter — SwiftWave

## Responsibility

This module bridges Android-specific networking APIs to the `swiftwave_core`
`PlatformAdapter` and `Discovery` traits via JNI (Java Native Interface).

## APIs Used

| API | Purpose | Min API Level |
|-----|---------|--------------|
| Wi-Fi Aware (NAN) | Ultra-low-latency peer discovery + direct data transfer without infrastructure | 26 (Android 8.0) |
| Wi-Fi Direct (P2P) | File transfer fallback when Wi-Fi Aware is unavailable | 14 (Android 4.0) |
| Bluetooth LE (BLE) | Advertisement / scan for discovery when Wi-Fi is unavailable | 21 (Android 5.0) |
| Wi-Fi Manager | AP-mode hotspot fallback | 26 |

## Planned Files

```
SwiftWaveDiscovery.kt       — Discovery interface + factory
WifiAwareDiscovery.kt   — Wi-Fi Aware NAN implementation  (TODO Phase 2)
WifiDirectDiscovery.kt  — Wi-Fi Direct P2P implementation  (TODO Phase 2)
BleDiscovery.kt         — BLE GATT advertisement          (TODO Phase 2)
SwiftWaveJni.kt             — JNI bridge to Rust swiftwave_ffi     (TODO Phase 2)
```

## Permissions Required

```xml
<!-- AndroidManifest.xml -->
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
<uses-permission android:name="android.permission.CHANGE_WIFI_STATE" />
<uses-permission android:name="android.permission.NEARBY_WIFI_DEVICES"
    android:usesPermissionFlags="neverForLocation" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.BLUETOOTH_SCAN"
    android:usesPermissionFlags="neverForLocation" />
<uses-permission android:name="android.permission.BLUETOOTH_ADVERTISE" />
<uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />
```

## References

- https://developer.android.com/develop/connectivity/wifi/wifi-aware
- https://developer.android.com/develop/connectivity/wifi/wifip2p
- https://developer.android.com/develop/connectivity/bluetooth/ble/ble-overview
