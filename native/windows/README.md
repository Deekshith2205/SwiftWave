# Windows Native Adapter — SwiftWave

## Responsibility

Provides Windows-specific peer discovery and transport integration for `swiftwave_core`.

## APIs Planned

| API | Purpose |
|-----|---------|
| DNS-SD / mDNS | Peer discovery on local Wi-Fi (via Windows DNS-SD APIs or `mdns-sd` Rust crate) |
| WinSock2 / UDP | Fallback broadcast discovery |
| Windows Firewall API | Register SwiftWave app exception |
| Windows Credential Store | Secure device keypair storage (via DPAPI) |

## TODO (Phase 2) — Files to create

```
swiftwave_discovery_win.rs  — mDNS + UDP broadcast discovery (Rust, platform feature-flag)
swiftwave_storage_win.rs    — DPAPI keypair storage
```

## Notes

- QUIC (quinn) binds to UDP sockets — no special Windows driver required.
- mDNS on Windows requires the Bonjour service or Windows 10 1703+ built-in mDNS.
- Use `cfg(target_os = "windows")` guards in swiftwave_core platform module.

## References

- https://learn.microsoft.com/en-us/windows/win32/dns/dns-service-discovery
- https://docs.rs/mdns-sd
