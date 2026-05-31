#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "=== Mario 2D Platformer Demo — Environment Bootstrap ==="
echo ""

# --- Step 1: Rust toolchain (rustup) ---
echo "[1/5] Rust toolchain (rustup)..."
if command -v rustup &>/dev/null; then
    echo "  Found: $(rustup --version)"
else
    echo "  ERROR: rustup not found."
    echo "  Install from https://rustup.rs (or https://win.rustup.rs/x86_64 on Windows)."
    echo "  Accept default options; MSVC toolchain is required for Macroquad."
    exit 1
fi
rustup update stable
rustup default stable

# --- Step 2: MSVC Build Tools (Macroquad requires link.exe on Windows) ---
echo "[2/5] MSVC Build Tools (Macroquad requirement)..."
if command -v link.exe &>/dev/null; then
    echo "  link.exe found: $(command -v link.exe)"
else
    echo "  WARNING: link.exe not found on PATH."
    echo "  Macroquad requires the MSVC toolchain on Windows."
    echo "  Download Visual Studio 2022 Build Tools from:"
    echo "  https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022"
    echo "  Select the 'Desktop development with C++' workload during install."
    echo "  After installation, re-run this script from a Developer Command Prompt."
fi

# --- Step 3: Coverage tool ---
echo "[3/5] Coverage tool (cargo-tarpaulin)..."
if cargo install --list 2>/dev/null | grep -q "cargo-tarpaulin v"; then
    echo "  Already installed."
else
    echo "  Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# --- Step 4: Build project ---
echo "[4/5] Build project (cargo build --release)..."
if [ -f "Cargo.toml" ]; then
    cargo build --release
    echo "  Build: OK"
else
    echo "  SKIP: No Cargo.toml found in project root."
fi

# --- Step 5: Verify ---
echo ""
echo "=== Environment Check ==="
echo "rustup:          $(rustup --version 2>/dev/null || echo 'not found')"
echo "rustc:           $(rustc --version 2>/dev/null || echo 'not found')"
echo "cargo:           $(cargo --version 2>/dev/null || echo 'not found')"

CLIPPY_VER=$(cargo clippy --version 2>/dev/null || echo 'available (bundled)')
echo "cargo-clippy:    ${CLIPPY_VER}"

TARP_VER=$(cargo tarpaulin --version 2>/dev/null || echo 'not installed')
echo "cargo-tarpaulin: ${TARP_VER}"

if [ -f "Cargo.toml" ]; then
    EDITION=$(grep 'edition' Cargo.toml | head -1 | sed 's/.*=\s*"\([^"]*\)".*/\1/')
    echo "cargo edition:   ${EDITION:-unknown}"
fi

echo ""
echo "Environment ready."
echo "  Build:   cargo build --release"
echo "  Test:    cargo test"
echo "  Cover:   cargo tarpaulin --out Xml --output-dir target/tarpaulin"
echo "  Lint:    cargo clippy -- -D warnings"
