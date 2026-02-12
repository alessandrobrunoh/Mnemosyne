# Mnemosyne Build Script
$ErrorActionPreference = "Stop"

Write-Host "--- Building Mnemosyne Binaries ---" -ForegroundColor Cyan

# 1. Ensure bin directory exists
if (!(Test-Path "bin")) {
    Write-Host "[*] Creating bin directory..." -ForegroundColor Blue
    New-Item -ItemType Directory -Path "bin" | Out-Null
}

# 2. Build binaries
Write-Host "[*] Compiling mnem-cli..." -ForegroundColor Blue
cargo build --release -p mnem-cli

Write-Host "[*] Compiling mnem-daemon..." -ForegroundColor Blue
cargo build --release -p mnem-daemon

# 3. Copy binaries
Write-Host "[*] Updating binaries in .\bin\..." -ForegroundColor Blue

$bins = @("mnem.exe", "mnem-daemon.exe")
foreach ($name in $bins) {
    $src = "target\release\$name"
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination "bin\$name" -Force
        Write-Host "[+] Success: $name updated" -ForegroundColor Green
    } else {
        Write-Host "[!] Warning: $src not found" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "Build complete!" -ForegroundColor Cyan
