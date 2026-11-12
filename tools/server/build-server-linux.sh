#!/bin/bash
# Собрать онлайн-сервер Blackwire под Linux (статический, musl — идёт на любом x86_64 Linux).
# Запуск из WSL в корне проекта:  bash tools/server/build-server-linux.sh
# Итог: server/deploy/blackwire-server (бинарник) — его и ставит install.sh.
set -e
cd "$(dirname "$0")/../.."
source "$HOME/.cargo/env" 2>/dev/null || true
rustup target add x86_64-unknown-linux-musl >/dev/null
export CARGO_TARGET_DIR="$HOME/bw-target"
export CC_x86_64_unknown_linux_musl=gcc
cargo build --release -p blackwire-server --target x86_64-unknown-linux-musl --jobs 2
cp "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/blackwire-server" server/deploy/blackwire-server
chmod 755 server/deploy/blackwire-server
file server/deploy/blackwire-server 2>/dev/null || true
ls -la server/deploy/blackwire-server
