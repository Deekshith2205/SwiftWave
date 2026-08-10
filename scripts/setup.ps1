# SwiftWave — Windows Developer Setup
# Installs Rust toolchain and checks Flutter prerequisites.

param(
    [switch]$SkipRust,
    [switch]$SkipFlutter
)

$ErrorActionPreference = "Stop"

Write-Host "=== SwiftWave Developer Setup (Windows) ===" -ForegroundColor Cyan
Write-Host ""

# ---------------------------------------------------------------------------
# Rust
# ---------------------------------------------------------------------------
if (-not $SkipRust) {
    if (Get-Command rustup -ErrorAction SilentlyContinue) {
        Write-Host "[OK] Rust already installed:" -ForegroundColor Green
        rustup --version
        cargo --version
    } else {
        Write-Host "[INFO] Installing Rust via rustup..." -ForegroundColor Yellow
        $tempFile = Join-Path $env:TEMP "rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $tempFile
        Start-Process -FilePath $tempFile -ArgumentList "-y" -Wait
        # Reload PATH
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" +
                    [System.Environment]::GetEnvironmentVariable("PATH", "User")
        Write-Host "[OK] Rust installed." -ForegroundColor Green
    }

    # Add Android targets
    Write-Host "[INFO] Adding Android Rust targets..." -ForegroundColor Yellow
    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
    Write-Host "[OK] Android targets added." -ForegroundColor Green

    # Install cargo-ndk for Android cross-compilation
    if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
        Write-Host "[INFO] Installing cargo-ndk..." -ForegroundColor Yellow
        cargo install cargo-ndk
    }
}

# ---------------------------------------------------------------------------
# Flutter
# ---------------------------------------------------------------------------
if (-not $SkipFlutter) {
    if (Get-Command flutter -ErrorAction SilentlyContinue) {
        Write-Host "[OK] Flutter already installed:" -ForegroundColor Green
        flutter --version
    } else {
        Write-Host "[WARN] Flutter not found in PATH." -ForegroundColor Yellow
        Write-Host "       Download from: https://flutter.dev/docs/get-started/install/windows"
    }

    Write-Host "[INFO] Running flutter doctor..." -ForegroundColor Yellow
    flutter doctor
}

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
Write-Host ""
Write-Host "=== Setup complete ===" -ForegroundColor Cyan
Write-Host "Next steps:"
Write-Host "  cargo build          # Build Rust core"
Write-Host "  cargo test --all     # Run Rust tests"
Write-Host "  cd apps\flutter_app"
Write-Host "  flutter pub get"
Write-Host "  flutter run -d windows"
