#!/bin/bash
# Cross-Platform Build Script with Automatic Process Cleanup
# Solves: File lock issues when rebuilding intent-api binary
# Usage: bash setup/rebuild_api.sh [--release]

set -e

echo "[REBUILD] Starting build with process cleanup..."

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*|MINGW*|MSYS*) MACHINE=Windows;;
    *)          MACHINE="UNKNOWN:${OS}"
esac

echo "[CLEANUP] Detected OS: ${MACHINE}"

# Kill cargo run processes
echo "[CLEANUP] Terminating cargo run processes..."
if [ "${MACHINE}" = "Windows" ]; then
    # Windows (Git Bash/MSYS2)
    ps aux | grep -E "cargo.*run.*intent-api" | grep -v grep | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true
else
    # Linux/Mac
    pkill -9 -f "cargo run.*intent-api" 2>/dev/null || true
fi

# Kill intent-api processes
echo "[CLEANUP] Terminating intent-api processes..."
if [ "${MACHINE}" = "Windows" ]; then
    ps aux | grep "intent-api" | grep -v grep | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true
else
    pkill -9 intent-api 2>/dev/null || true
fi

# Give OS time to release file locks
sleep 2

# Build based on argument
if [ "$1" = "--release" ]; then
    echo "[BUILD] Building release version..."
    cargo build --release --bin intent-api
else
    echo "[BUILD] Building debug version..."
    cargo build --bin intent-api
fi

echo "[SUCCESS] Build completed successfully"
