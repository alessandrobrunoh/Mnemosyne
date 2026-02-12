# Installation script for Mnemosyne LSP Server on Windows
# This script builds and installs mnem-lsp to your system

#Requires -RunAsAdministrator

# Color output helper functions
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Log-Info {
    Write-ColorOutput Cyan "[INFO] $args"
}

function Log-Success {
    Write-ColorOutput Green "[SUCCESS] $args"
}

function Log-Warning {
    Write-ColorOutput Yellow "[WARNING] $args"
}

function Log-Error {
    Write-ColorOutput Red "[ERROR] $args"
}

# Check if Rust is installed
function Test-RustInstallation {
    Log-Info "Checking Rust installation..."

    try {
        $rustVersion = rustc --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            Log-Success "Found $rustVersion"
            return $true
        }
    }
    catch {
        # Rust not found
    }

    Log-Error "Rust is not installed. Please install Rust from https://rustup.rs/"
    return $false
}

# Check if Mnemosyne daemon is installed
function Test-MnemosyneInstallation {
    Log-Info "Checking Mnemosyne installation..."

    try {
        $mnemdPath = Get-Command mnemd -ErrorAction Stop
        Log-Success "Found Mnemosyne daemon at $($mnemdPath.Source)"
        return $true
    }
    catch {
        Log-Warning "Mnemosyne daemon (mnemd) not found in PATH"
        Log-Info "You can still install mnem-lsp, but ensure mnemd is available when using the LSP"
        return $false
    }
}

# Build the LSP server
function Build-LspServer {
    Log-Info "Building mnem-lsp in release mode..."

    # Get project root (assuming script is in scripts/ subdirectory)
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $projectRoot = Split-Path -Parent $scriptDir

    Push-Location $projectRoot

    try {
        cargo build --release -p mnem-lsp
        if ($LASTEXITCODE -eq 0) {
            Log-Success "Build completed successfully"
            Pop-Location
            return $true
        }
        else {
            Log-Error "Build failed. Please check error messages above."
            Pop-Location
            return $false
        }
    }
    catch {
        Log-Error "Build failed: $_"
        Pop-Location
        return $false
    }
}

# Install binary to system
function Install-Binary {
    Log-Info "Installing mnem-lsp binary..."

    # Determine installation path
    $installDir = "C:\Program Files\Mnemosyne"
    $binaryName = "mnem-lsp.exe"

    # Get source binary path
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $projectRoot = Split-Path -Parent $scriptDir
    $sourceBinary = Join-Path $projectRoot "target\release\$binaryName"

    if (-not (Test-Path $sourceBinary)) {
        Log-Error "Binary not found at $sourceBinary"
        Log-Info "Did the build succeed?"
        return $false
    }

    # Create installation directory if it doesn't exist
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        Log-Info "Created installation directory: $installDir"
    }

    # Copy binary
    $destinationPath = Join-Path $installDir $binaryName
    Copy-Item -Path $sourceBinary -Destination $destinationPath -Force

    if (Test-Path $destinationPath) {
        Log-Success "Installed to $destinationPath"
        return $true
    }
    else {
        Log-Error "Failed to copy binary to $destinationPath"
        return $false
    }
}

# Add to system PATH
function Add-ToPath {
    Log-Info "Adding Mnemosyne to system PATH..."

    $installDir = "C:\Program Files\Mnemosyne"
    $pathEnv = [Environment]::GetEnvironmentVariable("Path", "Machine")

    if ($pathEnv -notlike "*$installDir*") {
        $newPath = "$pathEnv;$installDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
        Log-Success "Added $installDir to system PATH"
        Log-Warning "You may need to restart your terminal or IDE for PATH changes to take effect"
    }
    else {
        Log-Info "Mnemosyne already in system PATH"
    }
}

# Print Zed configuration instructions
function Show-ZedConfiguration {
    $mnemLspPath = "C:\Program Files\Mnemosyne\mnem-lsp.exe"

    Write-Host ""
    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host "Installation Complete!" -ForegroundColor Green
    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host ""

    Write-Host "Next steps to configure Zed IDE:" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "1. Open Zed settings (Ctrl+, or Settings -> Open Settings)"
    Write-Host ""
    Write-Host "2. Add the following to your settings.json:"
    Write-Host ""
    Write-Host "```json" -ForegroundColor Gray
    Write-Host "{" -ForegroundColor White
    Write-Host "  `"languages`": {" -ForegroundColor White
    Write-Host "    `"Rust`": {" -ForegroundColor White
    Write-Host "      `"language_servers`": [`"rust-analyzer`", `"mnem-lsp`"]" -ForegroundColor White
    Write-Host "    }" -ForegroundColor White
    Write-Host "  }," -ForegroundColor White
    Write-Host "  `"lsp`": {" -ForegroundColor White
    Write-Host "    `"mnem-lsp`": {" -ForegroundColor White
    Write-Host "      `"binary`": {" -ForegroundColor White
    Write-Host "        `"path`": `"$mnemLspPath`"," -ForegroundColor White
    Write-Host "        `"arguments`": []" -ForegroundColor White
    Write-Host "      }" -ForegroundColor White
    Write-Host "    }" -ForegroundColor White
    Write-Host "  }" -ForegroundColor White
    Write-Host "}" -ForegroundColor White
    Write-Host "```" -ForegroundColor Gray
    Write-Host ""

    Write-Host "3. For other languages, replace `"Rust`" with:" -ForegroundColor Yellow
    Write-Host "   - Python, TypeScript, JavaScript, Go, Java, C, C++, etc."
    Write-Host ""

    Write-Host "4. Start Mnemosyne daemon and watch your project:" -ForegroundColor Yellow
    Write-Host "   mnem start" -ForegroundColor Cyan
    Write-Host "   cd C:\path\to\your\project" -ForegroundColor Cyan
    Write-Host "   mnem watch" -ForegroundColor Cyan
    Write-Host ""

    Write-Host "5. Restart Zed to pick up the new LSP server" -ForegroundColor Yellow
    Write-Host ""

    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host "Usage:" -ForegroundColor Green
    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "- Hover over any symbol to see version count and last modification" -ForegroundColor White
    Write-Host "- Ctrl+Click or F12 to navigate between historical versions" -ForegroundColor White
    Write-Host ""

    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host "Troubleshooting:" -ForegroundColor Green
    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "- Check daemon status: " -NoNewline; Write-Host "mnem status" -ForegroundColor Cyan
    Write-Host "- Verify project is watched: " -NoNewline; Write-Host "mnem watch" -ForegroundColor Cyan
    Write-Host "- Check Zed logs: Ctrl+Shift+P -> 'zed: open log'"
    Write-Host "- Run with debug: " -NoNewline; Write-Host "`$env:RUST_LOG='mnem_lsp=debug'; mnem-lsp" -ForegroundColor Cyan
    Write-Host ""
}

# Uninstall function
function Uninstall-Binary {
    Log-Info "Uninstalling mnem-lsp..."

    $installDir = "C:\Program Files\Mnemosyne"

    if (Test-Path $installDir) {
        Remove-Item -Path $installDir -Recurse -Force
        Log-Success "Removed $installDir"
    }
    else {
        Log-Warning "Mnemosyne installation directory not found"
    }

    # Remove from PATH
    $pathEnv = [Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($pathEnv -like "*$installDir*") {
        $newPath = $pathEnv -replace [regex]::Escape(";$installDir"), ""
        [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
        Log-Success "Removed from system PATH"
    }
}

# Main installation process
function Main {
    Write-Host ""
    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host "Mnemosyne LSP Server Installer" -ForegroundColor Green
    Write-Host "=======================================" -ForegroundColor Cyan
    Write-Host ""

    # Parse command line arguments
    $command = if ($args.Count -gt 0) { $args[0] } else { "install" }

    switch ($command) {
        "uninstall" {
            Uninstall-Binary
            exit 0
        }
        "install" {
            # Run installation steps
            if (-not (Test-RustInstallation)) {
                exit 1
            }

            Test-MnemosyneInstallation  # Continue even if daemon not found

            if (-not (Build-LspServer)) {
                exit 1
            }

            if (-not (Install-Binary)) {
                exit 1
            }

            Add-ToPath
            Show-ZedConfiguration

            Log-Success "Installation completed successfully!"
            exit 0
        }
        default {
            Log-Error "Unknown command: $command"
            Write-Host "Usage: .\install.ps1 [install|uninstall]"
            exit 1
        }
    }
}

# Run main function
Main $args
