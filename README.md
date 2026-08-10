# SwiftWave

> **Privacy-first, offline peer-to-peer file sharing.**
> No internet. No cloud. No accounts.

---

## What is SwiftWave?

SwiftWave transfers files **directly** between nearby devices over Wi-Fi or Bluetooth —
without touching the internet, routing through servers, or requiring you to create an account.

Everything is encrypted end-to-end with modern cryptography (Noise protocol + ChaCha20-Poly1305).
Your files go straight from one device to another and nowhere else.

---

## Platform Support

| Platform | Status |
|----------|--------|
| Android 8.0+ | Phase 1 skeleton |
| Windows 10+ | Phase 1 skeleton |
| Linux (Ubuntu 20.04+) | Phase 1 skeleton |
| macOS 12+ | Phase 1 skeleton |

---

## Architecture

```
Flutter UI (Dart)
      ↓  Dart FFI
Rust Core (swiftwave_ffi → swiftwave_core)
      ↓
Transport / Security / File Engine
      ↓
Platform-specific discovery adapters
  Android: Wi-Fi Aware / Wi-Fi Direct / BLE
  Desktop: mDNS / DNS-SD / Bonjour / Avahi
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design.

---

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Flutter 3.44 · Dart 3.12 · Material 3 |
| Core engine | Rust · Tokio · async-trait |
| Transport | Quinn (QUIC) — Phase 2 |
| Security | Noise_XX · X25519 · ChaCha20-Poly1305 — Phase 2 |
| Hashing | BLAKE3 — Phase 2 |
| Compression | Zstandard (optional) — Phase 3 |
| Android native | Kotlin · Wi-Fi Aware · BLE |

---

## Repository Layout

```
swiftwave/
├── apps/
│   └── flutter_app/       Flutter application (Material 3 shell)
├── core/
│   └── swiftwave_core/        Rust core library (transport-agnostic)
├── packages/
│   └── swiftwave_ffi/         Rust C-ABI bridge for Dart FFI
├── native/
│   ├── android/           Kotlin Wi-Fi Aware / BLE adapter
│   ├── windows/           Windows mDNS adapter stub
│   ├── linux/             Linux Avahi adapter stub
│   └── macos/             macOS Bonjour adapter stub
├── docs/
│   ├── ARCHITECTURE.md    System design and data flow
│   ├── BUILD.md           Build instructions per platform
│   ├── ROADMAP.md         Phased development plan
│   ├── PLATFORM_MATRIX.md Feature × platform support table
│   └── SECURITY.md        Threat model and cryptographic design
├── tests/                 Integration test workspace
├── benchmarks/            Throughput and latency benchmarks
├── scripts/               Developer setup and build automation
├── Cargo.toml             Rust workspace root
└── README.md
```

---

## Quick Start

```bash
# Prerequisites: Rust (rustup.rs) + Flutter (flutter.dev)

# Build Rust core
cargo build

# Run Rust tests
cargo test --all

# Build Flutter app (Windows)
cd apps/flutter_app
flutter pub get
flutter run -d windows
```

Full instructions: [docs/BUILD.md](docs/BUILD.md)

---

## Design Principles

- **Zero cloud** — no servers are involved in any transfer.
- **Zero accounts** — no registration, no login, no email.
- **Zero metadata leakage** — no analytics, no telemetry.
- **Small footprint** — optimised for low RAM, low CPU, and small binary size.
- **Modular** — every component is replaceable via traits; no tight coupling.
- **Testable** — stub implementations for every backend; tests run without hardware.

---

## Contributing

See [docs/BUILD.md](docs/BUILD.md) for the development environment setup.
Follow the phased roadmap in [docs/ROADMAP.md](docs/ROADMAP.md).

---

## License

MIT OR Apache-2.0 — see `LICENSE-MIT` and `LICENSE-APACHE`.

---

## Security

To report a vulnerability, see [docs/SECURITY.md](docs/SECURITY.md).
