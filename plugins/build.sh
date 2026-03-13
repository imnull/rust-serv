#!/bin/bash

# Build all example plugins

set -e

echo "🔨 Building example plugins..."

# Build add-header plugin
echo "Building add-header plugin..."
cd examples/add-header
cargo build --target wasm32-unknown-unknown --release
cd ../..

# Build rate-limiter plugin
echo "Building rate-limiter plugin..."
cd examples/rate-limiter
cargo build --target wasm32-unknown-unknown --release
cd ../..

echo "✅ All plugins built!"
echo ""
echo "Output files:"
ls -lh examples/*/target/wasm32-unknown-unknown/release/*.wasm
