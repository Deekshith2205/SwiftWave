#!/usr/bin/env bash
# SwiftWave — Linux/macOS Developer Setup
# Installs Rust toolchain and checks Flutter prerequisites.

set -euo pipefail

echo "=== SwiftWave Developer Setup (Linux/macOS) ==="

# ---------------------------------------------------------------------------
# Rust
# ---------------------------------------------------------------------------
if command -v rustup &>/dev/null; then
    echo "[OK] Rust already installed: $(rustup --version)"
else
    echo "[INFO] Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    echo "[OK] Rust installed."
fi

# Android targets
echo "[INFO] Adding Android Rust targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
echo "[OK] Android targets added."

# cargo-ndk
if ! command -v cargo-ndk &>/dev/null; then
    echo "[INFO] Installing cargo-ndk..."
    cargo install cargo-ndk
fi

# ---------------------------------------------------------------------------
# Platform-specific deps
# ---------------------------------------------------------------------------
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "[INFO] Installing Linux native dependencies..."
    if command -v apt-get &>/dev/null; then
        sudo apt-get update -q
        sudo apt-get install -y \
            build-essential clang cmake pkg-config \
            libavahi-client-dev libsecret-1-dev \
            libasound2-dev libpulse-dev
    else
        echo "[WARN] apt-get not found. Please install build-essential, clang, cmake, pkg-config, libavahi-client-dev manually."
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "[INFO] macOS detected — Xcode tools required."
    if ! xcode-select -p &>/dev/null; then
        echo "[INFO] Installing Xcode Command Line Tools..."
        xcode-select --install
    else
        echo "[OK] Xcode CLT already installed."
    fi
fi

# ---------------------------------------------------------------------------
# Flutter
# ---------------------------------------------------------------------------
if command -v flutter &>/dev/null; then
    echo "[OK] Flutter already installed: $(flutter --version | head -1)"
    flutter doctor
else
    echo "[WARN] Flutter not found. Install from: https://flutter.dev/docs/get-started/install"
fi

echo ""
echo "=== Setup complete ==="
echo "Next steps:"
echo "  cargo build          # Build Rust core"
echo "  cargo test --all     # Run Rust tests"
echo "  cd apps/flutter_app"
echo "  flutter pub get"
echo "  flutter run -d linux   # or -d macos"
