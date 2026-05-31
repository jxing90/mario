$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "=== Mario 2D Platformer Demo — Environment Bootstrap ==="
Write-Host ""

# --- Step 1: Rust toolchain (rustup) ---
Write-Host "[1/5] Rust toolchain (rustup)..."
if (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Host "  Found: $(rustup --version)"
} else {
    Write-Host "  ERROR: rustup not found."
    Write-Host "  Install from https://rustup.rs (or https://win.rustup.rs/x86_64 on Windows)."
    Write-Host "  Accept default options; MSVC toolchain is required for Macroquad."
    exit 1
}
rustup update stable
rustup default stable

# --- Step 2: MSVC Build Tools (Macroquad requires link.exe on Windows) ---
Write-Host "[2/5] MSVC Build Tools (Macroquad requirement)..."
if (Get-Command link.exe -ErrorAction SilentlyContinue) {
    Write-Host "  link.exe found: $(Get-Command link.exe | Select-Object -ExpandProperty Source)"
} else {
    Write-Host "  WARNING: link.exe not found on PATH."
    Write-Host "  Macroquad requires the MSVC toolchain on Windows."
    Write-Host "  Download Visual Studio 2022 Build Tools from:"
    Write-Host "  https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022"
    Write-Host "  Select the 'Desktop development with C++' workload during install."
    Write-Host "  After installation, re-run this script from a Developer PowerShell prompt."
}

# --- Step 3: Coverage tool ---
Write-Host "[3/5] Coverage tool (cargo-tarpaulin)..."
$tarpaulinInstalled = cargo install --list 2>$null | Select-String "cargo-tarpaulin v"
if ($tarpaulinInstalled) {
    Write-Host "  Already installed."
} else {
    Write-Host "  Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
}

# --- Step 4: Build project ---
Write-Host "[4/5] Build project (cargo build --release)..."
if (Test-Path "Cargo.toml") {
    cargo build --release
    Write-Host "  Build: OK"
} else {
    Write-Host "  SKIP: No Cargo.toml found in project root."
}

# --- Step 5: Verify ---
Write-Host ""
Write-Host "=== Environment Check ==="

try { Write-Host "rustup:          $(rustup --version 2>$null)" }
catch { Write-Host "rustup:          not found" }

try { Write-Host "rustc:           $(rustc --version 2>$null)" }
catch { Write-Host "rustc:           not found" }

try { Write-Host "cargo:           $(cargo --version 2>$null)" }
catch { Write-Host "cargo:           not found" }

try {
    $clippyVer = cargo clippy --version 2>$null
    Write-Host "cargo-clippy:    $clippyVer"
} catch {
    Write-Host "cargo-clippy:    available (bundled)"
}

try {
    $tarpVer = cargo tarpaulin --version 2>$null
    Write-Host "cargo-tarpaulin: $tarpVer"
} catch {
    Write-Host "cargo-tarpaulin: not installed"
}

if (Test-Path "Cargo.toml") {
    $cargoToml = Get-Content "Cargo.toml" -Raw
    if ($cargoToml -match 'edition\s*=\s*"([^"]*)"') {
        Write-Host "cargo edition:   $($Matches[1])"
    }
}

Write-Host ""
Write-Host "Environment ready."
Write-Host "  Build:   cargo build --release"
Write-Host "  Test:    cargo test"
Write-Host "  Cover:   cargo tarpaulin --out Xml --output-dir target/tarpaulin"
Write-Host "  Lint:    cargo clippy -- -D warnings"
