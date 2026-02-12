# Mnemosyne Uninstallation Script for Windows
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
Write-Host "[*] Removing files in $InstallDir..." -ForegroundColor Blue
if (Test-Path $InstallDir) {
    try {
        Remove-Item -Path $InstallDir -Recurse -Force
        Write-Host "[+] Successfully removed $InstallDir." -ForegroundColor Green
    } catch {
        Write-Host "[!] Warning: Could not remove $InstallDir completely. Some files might be in use." -ForegroundColor Yellow
        Write-Host "    Try closing all terminals and running this again." -ForegroundColor Gray
    }
} else {
    Write-Host "[-] $InstallDir not found." -ForegroundColor Gray
}

Write-Host ""
Write-Host "Mnemosyne has been uninstalled." -ForegroundColor Cyan
