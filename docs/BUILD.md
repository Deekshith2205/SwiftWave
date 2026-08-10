# SwiftWave — Build Instructions

## Prerequisites

### All Platforms
| Tool | Version | Install |
|------|---------|---------|
| Rust + Cargo | stable (≥ 1.80) | https://rustup.rs |
| Flutter SDK | ≥ 3.44 | https://flutter.dev/docs/get-started/install |
| Dart SDK | ≥ 3.12 (bundled with Flutter) | — |

### Android
| Tool | Version |
|------|---------|
| Android Studio | Koala or later |
| Android SDK | API 34 |
| NDK | r26+ |
| Kotlin | 1.9+ |

### Windows
| Tool | Note |
|------|------|
| Visual Studio 2022 | C++ Desktop workload |
| Windows SDK | 10.0.19041+ |

### Linux
| Tool | Install |
|------|---------|
| GCC / Clang | `sudo apt install build-essential clang` |
| CMake | `sudo apt install cmake` |
| pkg-config | `sudo apt install pkg-config` |
| libavahi-client-dev | `sudo apt install libavahi-client-dev` |

### macOS
| Tool | Note |
|------|------|
| Xcode | 15+ |
| Command Line Tools | `xcode-select --install` |

---

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/your-org/swiftwave.git
cd swiftwave

# 2. Install Rust toolchain
rustup target add aarch64-linux-android   # for Android
rustup target add x86_64-pc-windows-msvc  # for Windows (on Windows)

# 3. Build the Rust core (all crates)
cargo build

# 4. Run Rust tests
cargo test --all

# 5. Build the Flutter app (Windows example)
cd apps/flutter_app
flutter pub get
flutter run -d windows
```

---

## Rust Build

```bash
# Development build
cargo build

# Release build (optimised, stripped)
cargo build --release

# Run all tests
cargo test --all

# Run with coverage (requires cargo-llvm-cov)
cargo llvm-cov --all

# Lint
cargo clippy --all -- -D warnings

# Format
cargo fmt --all
```

### Output locations
| Platform | Library |
|----------|---------|
| Windows | `target/release/swiftwave_ffi.dll` |
| Linux | `target/release/libswiftwave_ffi.so` |
| macOS | `target/release/libswiftwave_ffi.dylib` |
| Android arm64 | `target/aarch64-linux-android/release/libswiftwave_ffi.so` |

---

## Flutter Build

```bash
cd apps/flutter_app

# Get dependencies
flutter pub get

# Analyse (no issues expected)
flutter analyze

# Run tests
flutter test

# Run on Windows desktop
flutter run -d windows

# Run on Linux desktop
flutter run -d linux

# Run on macOS
flutter run -d macos

# Build release APK (Android)
flutter build apk --release

# Build release MSIX (Windows)
flutter build windows --release
```

---

## Android Cross-compilation

```bash
# Add Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Install cargo-ndk
cargo install cargo-ndk

# Build for all Android architectures
cargo ndk -t armeabi-v7a -t arm64-v8a -t x86_64 \
  -o apps/flutter_app/android/app/src/main/jniLibs \
  build --release -p swiftwave_ffi

# Then build the Flutter APK
cd apps/flutter_app
flutter build apk --release
```

---

## Automated Scripts

| Script | Platform | Action |
|--------|----------|--------|
| `scripts/setup.ps1` | Windows | Install Rust, set up environment |
| `scripts/setup.sh` | Linux/macOS | Install Rust, set up environment |
| `scripts/build_all.ps1` | Windows | Build Rust + Flutter |

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SWIFTWAVE_LOG` | `warn` | Rust tracing log level (`error`, `warn`, `info`, `debug`, `trace`) |
| `SWIFTWAVE_TRANSPORT` | `quic` | Force transport backend (Phase 2) |
| `SWIFTWAVE_DISCOVERY` | `auto` | Force discovery backend (Phase 2) |
