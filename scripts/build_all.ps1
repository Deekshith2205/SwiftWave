# SwiftWave — Windows All-in-One Build Script
# Builds Rust core and Flutter Windows app in sequence.

param(
    [ValidateSet("debug", "release")]
    [string]$Config = "debug"
)

$ErrorActionPreference = "Stop"
$StartTime = Get-Date

Write-Host "=== SwiftWave Build ($Config) ===" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# Rust build
# ---------------------------------------------------------------------------
Write-Host ""
Write-Host "--- Building Rust workspace ---" -ForegroundColor Yellow

$cargoArgs = @("build", "--workspace")
if ($Config -eq "release") { $cargoArgs += "--release" }

& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { throw "Rust build failed" }
Write-Host "[OK] Rust build succeeded." -ForegroundColor Green

# ---------------------------------------------------------------------------
# Rust tests
# ---------------------------------------------------------------------------
Write-Host ""
Write-Host "--- Running Rust tests ---" -ForegroundColor Yellow
& cargo test --all
if ($LASTEXITCODE -ne 0) { throw "Rust tests failed" }
Write-Host "[OK] All Rust tests passed." -ForegroundColor Green

# ---------------------------------------------------------------------------
# Flutter build
# ---------------------------------------------------------------------------
Write-Host ""
Write-Host "--- Building Flutter (Windows) ---" -ForegroundColor Yellow
Push-Location "apps\flutter_app"
try {
    flutter pub get
    flutter analyze
    if ($Config -eq "release") {
        flutter build windows --release
    } else {
        Write-Host "[INFO] Skipping flutter build in debug mode (use 'flutter run -d windows' instead)."
    }
    Write-Host "[OK] Flutter step succeeded." -ForegroundColor Green
} finally {
    Pop-Location
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
$Duration = (Get-Date) - $StartTime
Write-Host ""
Write-Host "=== Build complete in $($Duration.TotalSeconds.ToString('F1'))s ===" -ForegroundColor Cyan
