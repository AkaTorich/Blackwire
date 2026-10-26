#!/usr/bin/env bash
# Сборка Linux-версии Blackwire внутри WSL/Linux.
# Системные пакеты ставятся отдельно (нужен sudo):
#   sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev build-essential \
#     curl file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
# Rust и Tauri CLI этот скрипт при нужде поставит сам (без sudo).
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
# отдельная папка сборки на Linux-диске: быстрее и не мешает Windows-target
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/bw-target}"
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env" || true

ensure_toolchain() {
  if ! command -v rustup >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    . "$HOME/.cargo/env"
  fi
  rustup default stable >/dev/null
  if ! cargo tauri --version >/dev/null 2>&1; then
    cargo install tauri-cli --version '^2.0' --locked
  fi
  echo "toolchain: $(cargo --version) / $(cargo tauri --version)"
}

build() {
  cd "$HERE/app"
  cargo tauri build --bundles deb -- --jobs 2
  echo "BUILD_OK"
  local deb bin
  deb="$(ls "$CARGO_TARGET_DIR"/release/bundle/deb/*.deb 2>/dev/null | head -1 || true)"
  if [ -n "$deb" ]; then
    cp "$deb" "$HERE/"
    echo "COPIED_DEB=$(basename "$deb")"
  else
    echo "DEB_NOT_FOUND"
  fi
  bin="$(ls "$CARGO_TARGET_DIR"/release/[Bb]lackwire 2>/dev/null | head -1 || true)"
  if [ -n "$bin" ]; then
    cp "$bin" "$HERE/blackwire-linux"
    echo "COPIED_BIN=blackwire-linux"
  fi
}

case "${1:-all}" in
  toolchain) ensure_toolchain ;;
  build) build ;;
  all) ensure_toolchain; build ;;
esac
