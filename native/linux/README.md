# Linux Native Adapter — SwiftWave

## Responsibility

Provides Linux-specific peer discovery using Avahi (mDNS/DNS-SD daemon).

## APIs Planned

| API | Purpose |
|-----|---------|
| Avahi D-Bus API | mDNS peer discovery (`avahi-daemon` must be running) |
| `mdns-sd` crate (Rust) | Alternative pure-Rust mDNS (no daemon required) |
| BlueZ D-Bus API | BLE advertisement / scan |
| Netlink | Network interface monitoring |

## TODO (Phase 2) — Files to create

```
swiftwave_discovery_linux.rs  — Avahi or mdns-sd discovery
swiftwave_storage_linux.rs    — libsecret / keyring keypair storage
```

## Notes

- `#[cfg(target_os = "linux")]` guards required in swiftwave_core platform module.
- Flatpak packaging will require appropriate portal permissions.
- AppArmor / SELinux profiles may need updating for socket access.

## References

- https://avahi.org/doxygen/html/
- https://docs.rs/mdns-sd
- https://wiki.bluez.org/
