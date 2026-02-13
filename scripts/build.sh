#!/bin/bash
# Mnemosyne Build Script

set -e

echo "Building Mnemosyne..."

# Build release binaries
cargo build --release --package mnem-cli --package mnem-daemon

# Create bin directory if it doesn't exist
mkdir -p bin

# Copy binaries
cp target/release/mnem bin/mnem
cp target/release/mnem-daemon bin/mnem-daemon

echo "Done! Binaries in bin/"
