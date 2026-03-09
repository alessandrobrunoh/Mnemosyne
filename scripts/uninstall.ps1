# Mnemosyne Uninstallation Script for Windows
param (
    [switch]$purge
)
$ErrorActionPreference = "Continue"

Write-Host "--- Mnemosyne Uninstallation ---" -ForegroundColor Cyan

# 1. Stop processes
Write-Host "[*] Stopping Mnemosyne processes..." -ForegroundColor Blue
Stop-Process -Name "mnem" -ErrorAction SilentlyContinue
Stop-Process -Name "mnem-daemon" -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# 2. Remove from PATH
Write-Host "[*] Removing from PATH..." -ForegroundColor Blue
$InstallDir = "$env:USERPROFILE\.mnemosyne"
$BinDir = "$InstallDir\bin"
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($CurrentPath -like "*$BinDir*") {
    $PathArray = $CurrentPath.Split(';')
    $NewPathArray = $PathArray | Where-Object { $_ -ne $BinDir -and $_ -ne "$BinDir" -and $_ -notmatch "\.mnemosyne\bin" }
    $NewPath = $NewPathArray -join ';'
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = $NewPath
    Write-Host "[+] PATH updated." -ForegroundColor Green
} else {
    Write-Host "[-] $BinDir not found in User PATH." -ForegroundColor Gray
}

# 3. Remove files
if ($purge) {
    Write-Host "[*] Purging all files in $InstallDir..." -ForegroundColor Blue
    if (Test-Path $InstallDir) {
        try {
            Remove-Item -Path $InstallDir -Recurse -Force
            Write-Host "[+] Successfully removed all Mnemosyne data." -ForegroundColor Green
        } catch {
            Write-Host "[!] Warning: Could not remove $InstallDir completely. Some files might be in use." -ForegroundColor Yellow
        }
    } else {
        Write-Host "[-] $InstallDir not found." -ForegroundColor Gray
    }
} else {
    Write-Host "[*] Removing binaries in $BinDir..." -ForegroundColor Blue
    if (Test-Path $BinDir) {
        try {
            Remove-Item -Path $BinDir -Recurse -Force
            Write-Host "[+] Successfully removed binaries. Configuration and history preserved in $InstallDir." -ForegroundColor Green
        } catch {
            Write-Host "[!] Warning: Could not remove $BinDir completely." -ForegroundColor Yellow
        }
    } else {
        Write-Host "[-] $BinDir not found." -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "Mnemosyne has been uninstalled." -ForegroundColor Cyan
