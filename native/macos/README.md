# macOS Native Adapter — SwiftWave

## Responsibility

Provides macOS-specific peer discovery using Bonjour (mDNS/DNS-SD).

## APIs Planned

| API | Purpose |
|-----|---------|
| Bonjour / `dnssd` | Peer discovery — built into macOS, no daemon required |
| `Network.framework` | Modern async networking (QUIC support in macOS 12+) |
| CoreBluetooth | BLE advertisement / scan |
| Keychain Services | Secure device keypair storage |

## TODO (Phase 2) — Files to create

```
swiftwave_discovery_macos.rs  — Bonjour / dnssd discovery
swiftwave_storage_macos.rs    — Keychain keypair storage (via security-framework crate)
```

## Notes

- macOS 12+ has native QUIC via `Network.framework` — can supplement Quinn.
- Sandboxed apps (Mac App Store) need `com.apple.security.network.client` entitlement.
- `#[cfg(target_os = "macos")]` guards required.

## References

- https://developer.apple.com/documentation/dnssd
- https://developer.apple.com/documentation/network
- https://docs.rs/security-framework
