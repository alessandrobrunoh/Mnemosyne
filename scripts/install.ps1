# Mnemosyne Installation Script for Windows
$ErrorActionPreference = "Stop"

# Use UTF8 for emojis and better text
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "
  ███╗   ███╗███╗   ██╗███████╗███╗   ███╗
  ████╗ ████║████╗  ██║██╔════╝████╗ ████║
  ██╔████╔██║██╔██╗ ██║█████╗  ██╔████╔██║
  ██║╚██╔╝██║██║╚██╗██║██╔══╝  ██║╚██╔╝██║
  ██║ ╚═╝ ██║██║ ╚████║███████╗██║ ╚═╝ ██║
  ╚═╝     ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚═╝
" -ForegroundColor Cyan

Write-Host "--- Mnemosyne Quick Installation ---" -ForegroundColor Cyan

# 1. Configuration
$RepoUrl = "https://github.com/alessandrobrunoh/Mnemosyne/raw/main/bin"
$UserHome = $env:USERPROFILE
if (!$UserHome) { $UserHome = $HOME }
$InstallDir = "$UserHome\.mnemosyne"
$BinDir = "$InstallDir\bin"

Write-Host "[*] Target directory: $BinDir" -ForegroundColor Blue

# Create directories
if (!(Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

# 2. Stop running processes
Write-Host "[*] Stopping existing Mnemosyne processes..." -ForegroundColor Blue
Stop-Process -Name "mnem" -ErrorAction SilentlyContinue
Stop-Process -Name "mnem-daemon" -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# 3. Download Binaries
$binaries = @("mnem.exe", "mnem-daemon.exe")

foreach ($bin in $binaries) {
    $targetFile = "$BinDir\$bin"
    $url = "$RepoUrl/$bin"
    
    Write-Host "[-] Downloading $bin..." -ForegroundColor Gray
    try {
        # Check if local bin exists (if running from repo)
        if (Test-Path ".\bin\$bin") {
            Copy-Item ".\bin\$bin" $targetFile -Force
            Write-Host "    [Local copy used]" -ForegroundColor DarkGray
        } else {
            Invoke-WebRequest -Uri $url -OutFile $targetFile -UseBasicParsing -ErrorAction Stop
        }
    } catch {
        Write-Host "[!] Error: Could not download or copy $bin" -ForegroundColor Red
        Write-Host "    Please ensure you have internet access or the binaries exist in $RepoUrl" -ForegroundColor Gray
        exit 1
    }
}

Write-Host "[+] Binaries installed successfully." -ForegroundColor Green

# 4. Update PATH
Write-Host "[*] Configuring system PATH..." -ForegroundColor Blue
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($CurrentPath -split ';' -notcontains $BinDir -and $CurrentPath -split ';' -notcontains "$BinDir\") {
    $NewPath = "$CurrentPath;$BinDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = "$env:Path;$BinDir"
    Write-Host "[+] PATH updated. You can now use 'mnem' from any terminal." -ForegroundColor Green
} else {
    Write-Host "[+] $BinDir is already in PATH." -ForegroundColor Gray
}

# 5. Finalize
Write-Host ""
Write-Host "Mnemosyne is ready!" -ForegroundColor Green
Write-Host "---------------------------------------------------" -ForegroundColor Cyan
Write-Host "Next steps:" -ForegroundColor White
Write-Host "1. Close and reopen your terminal (PowerShell, CMD, etc.)" -ForegroundColor Gray
Write-Host "2. Type 'mnem on' to launch the background engine" -ForegroundColor Gray
Write-Host "3. Type 'mnem track' in your project folder to start tracking" -ForegroundColor Gray
Write-Host "---------------------------------------------------" -ForegroundColor Cyan
Write-Host "To uninstall, run: mnem off" -ForegroundColor DarkGray
