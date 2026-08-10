# SwiftWave — Platform Feature Matrix

## Supported Platforms

| Feature | Android | Windows | Linux | macOS |
|---------|:-------:|:-------:|:-----:|:-----:|
| **Build status** | Phase 1 skeleton | Phase 1 skeleton | Phase 1 skeleton | Phase 1 skeleton |
| **Min version** | Android 8.0 (API 26) | Windows 10 | Ubuntu 20.04 | macOS 12 |
| **Architecture** | arm64-v8a, x86_64 | x86_64 | x86_64, aarch64 | x86_64, Apple Silicon |

---

## Discovery Methods

| Method | Android | Windows | Linux | macOS |
|--------|:-------:|:-------:|:-----:|:-----:|
| Wi-Fi Aware (NAN) | ✅ Phase 2 | ❌ N/A | ❌ N/A | ❌ N/A |
| Wi-Fi Direct (P2P) | ✅ Phase 2 (fallback) | ❌ N/A | ❌ N/A | ❌ N/A |
| BLE GATT | ✅ Phase 2 (fallback) | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |
| mDNS / DNS-SD | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |
| UDP broadcast | 🔜 Phase 3 | 🔜 Phase 3 | 🔜 Phase 3 | 🔜 Phase 3 |

---

## Transport

| Transport | Android | Windows | Linux | macOS |
|-----------|:-------:|:-------:|:-----:|:-----:|
| QUIC (Quinn) | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |
| TCP (fallback) | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |

---

## Security

| Feature | Android | Windows | Linux | macOS |
|---------|:-------:|:-------:|:-----:|:-----:|
| Noise_XX handshake | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |
| ChaCha20-Poly1305 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |
| AES-256-GCM | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |
| TOFU key pinning | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 | 🔜 Phase 2 |
| Secure key storage | Android Keystore | DPAPI | libsecret | Keychain |

---

## File Engine

| Feature | All Platforms |
|---------|:------------:|
| BLAKE3 integrity | 🔜 Phase 2 |
| 256 KiB chunking | 🔜 Phase 2 |
| Resumable transfers | 🔜 Phase 2 |
| Zstd compression | 🔜 Phase 3 |
| Parallel streams | 🔜 Phase 3 |
| Directory transfer | 🔜 Phase 3 |

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and tested |
| 🔜 | Planned (phase noted) |
| ❌ | Not applicable / not planned |
