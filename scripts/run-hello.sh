#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Construindo cápsula hello (wasm32-wasi)..."
(
  cd "$ROOT_DIR"
  rustup target add wasm32-wasi >/dev/null 2>&1 || true
  cargo build -p hello-capsule --target wasm32-wasi
)

echo "==> Executando cápsula via runtime..."
cargo run -p caeles-runtime -- run --manifest "$ROOT_DIR/capsules/hello-capsule/manifest.json" "$@"
