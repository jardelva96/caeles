#!/usr/bin/env bash
# Quick start script for CAELES
# This script sets up and runs the hello-capsule example

set -e

echo "🚀 CAELES Quick Start"
echo "===================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust não encontrado. Instale via: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust encontrado: $(rustc --version)"

# Install wasm32-unknown-unknown target
echo ""
echo "📦 Instalando target wasm32-unknown-unknown..."
rustup target add wasm32-unknown-unknown

# Build runtime
echo ""
echo "🔨 Compilando runtime..."
cargo build --release --package caeles-runtime

# Build capsules
echo ""
echo "🔨 Compilando cápsulas de exemplo..."
cargo build --target wasm32-unknown-unknown --release --package hello-capsule --package logger-capsule

# List available capsules
echo ""
echo "📋 Cápsulas disponíveis:"
./target/release/caeles-runtime --list

# Run hello-capsule
echo ""
echo "🎉 Executando hello-capsule..."
echo "================================"
./target/release/caeles-runtime --capsule-id com.caeles.example.hello

echo ""
echo "================================"
echo "✅ Pronto! CAELES está funcionando."
echo ""
echo "Próximos passos:"
echo "  - Execute: ./target/release/caeles-runtime --list"
echo "  - Veja exemplos em: capsules/"
echo "  - Leia: README.md"
echo "  - Use make para facilitar: make help"
