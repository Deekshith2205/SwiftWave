# SwiftWave — Development Roadmap

## Phase 1 — Architecture & Skeleton ✅ (current)

**Goal**: Establish the project structure, build system, and interfaces.

- [x] Repository layout (`apps/`, `core/`, `packages/`, `native/`, `docs/`)
- [x] Rust workspace (`swiftwave_core` + `swiftwave_ffi`)
- [x] All core module traits: identity, discovery, security, transport, file_engine, storage, platform
- [x] Stub implementations for testing
- [x] Flutter app shell (Material 3, GoRouter, Riverpod)
- [x] Dart FFI bridge skeleton
- [x] Native adapter stubs (Android Kotlin, Windows, Linux, macOS)
- [x] Integration test scaffold
- [x] Documentation: architecture, roadmap, build, platform matrix, security

---

## Phase 2 — Core Transport & Security

**Goal**: Real encrypted peer-to-peer data transfer.

### Cryptography & Identity
- [ ] X25519 keypair generation using `x25519-dalek`
- [ ] Persist keypair to platform secure storage
- [ ] Noise_XX handshake using `snow` crate
- [ ] ChaCha20-Poly1305 session encryption using `chacha20poly1305`
- [ ] AES-256-GCM fallback (hardware-accelerated)
- [ ] TOFU peer key pinning (SQLite via `rusqlite`)

### Transport
- [ ] QUIC transport using `quinn 0.11`
- [ ] TCP fallback transport
- [ ] Stream multiplexing (one stream per chunk batch)
- [ ] Zero-copy chunk pipeline using `bytes::Bytes`

### Discovery
- [ ] mDNS discovery using `mdns-sd` (Windows, Linux, macOS)
- [ ] Android Wi-Fi Aware (Kotlin, JNI bridge)
- [ ] Android Wi-Fi Direct fallback (Kotlin, JNI bridge)
- [ ] BLE GATT advertisement using `btleplug`

### File Engine
- [ ] BLAKE3 streaming hasher
- [ ] 256 KiB chunk split/reassemble
- [ ] Transfer state machine (Pending → Active → Paused → Done | Failed)
- [ ] Resumable transfers (chunk bitmap)

### FFI Bridge
- [ ] `cbindgen` header auto-generation
- [ ] `ffigen` Dart binding auto-generation
- [ ] Async progress callback registration
- [ ] Handle-based API (replace raw pointer stubs)

---

## Phase 3 — File Engine Optimisations

**Goal**: Maximum throughput, minimum resource usage.

- [ ] Optional Zstandard compression per file type
- [ ] Parallel chunk streaming over multiple QUIC streams
- [ ] Bandwidth throttling / QoS
- [ ] Large file support (> 4 GB)
- [ ] Directory transfer (zip-less archive of folder)
- [ ] Transfer queue management

---

## Phase 4 — Flutter UI & UX

**Goal**: Polished, intuitive UI across all platforms.

- [ ] Device discovery list with real-time updates
- [ ] File picker (file + folder + multiple selection)
- [ ] Transfer progress cards with pause/cancel
- [ ] Transfer history
- [ ] Device detail sheet (TOFU key fingerprint display)
- [ ] Settings screen (device name, storage path, cipher preference)
- [ ] Onboarding flow (permissions, device name)
- [ ] Adaptive layout (mobile, tablet, desktop)
- [ ] Accessibility (semantic labels, contrast ratios)

---

## Phase 5 — Polish & Release

**Goal**: Production-ready, audited, packaged releases.

- [ ] Security audit (Noise handshake, key storage)
- [ ] Penetration testing of discovery and transport layers
- [ ] Android: Wi-Fi Aware SDK integration tests on real hardware
- [ ] Platform packaging: APK/AAB, MSIX, Flatpak, DMG
- [ ] CI/CD pipeline (GitHub Actions: Rust + Flutter per platform)
- [ ] Crash-free release (no unwinding in release builds)
- [ ] Privacy policy document
- [ ] End-to-end Instrumentation tests

---

## Out of Scope (by design)

- ❌ Internet connectivity
- ❌ Cloud storage or relay servers
- ❌ User accounts or authentication
- ❌ End-to-end key escrow
- ❌ iOS (requires separate native implementation — future phase)
