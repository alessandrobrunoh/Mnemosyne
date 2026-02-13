# Mnemosyne Build Script (Windows)

Write-Host "Building Mnemosyne..." -ForegroundColor Cyan

# Build release binaries
cargo build --release --package mnem-cli --package mnem-daemon

# Create bin directory if it doesn't exist
if (-not (Test-Path bin)) {
    New-Item -ItemType Directory -Path bin | Out-Null
}

# Copy binaries
Copy-Item target/release/mnem.exe bin/
Copy-Item target/release/mnem-daemon.exe bin/

Write-Host "Done! Binaries in bin/" -ForegroundColor Green
