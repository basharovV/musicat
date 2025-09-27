#!/usr/bin/env sh
set -euo pipefail

# --- Parse args / env ---
TARGET_TRIPLE="${TAURI_ENV_TARGET_TRIPLE-}"
while [ $# -gt 0 ]; do
  case "$1" in
    --target) shift; TARGET_TRIPLE="$1" ;;
    *) echo "Usage: $0 [--target <triple>]"; exit 1 ;;
  esac
  shift
done
[ -n "$TARGET_TRIPLE" ] || { echo "❌ No target triple (use --target or TAURI_ENV_TARGET_TRIPLE)"; exit 1; }

# --- Script dir ---
cd "$(dirname "$0")"

# --- Build ---
cd ./portable-vocal-remover/pvr && cargo build -r -p pvr
cd ..
# --- Rename ---
BIN=./target/release/pvr
NEW=./target/release/pvr-"$TARGET_TRIPLE"
mv "$BIN" "$NEW"
echo "✅ Built: $NEW"