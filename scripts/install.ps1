<#
.SYNOPSIS
    Mnemosyne Installation Script for Windows
.DESCRIPTION
    Downloads and installs the latest Mnemosyne release from GitHub
#>

#Requires -Version 5.1

# Set error action preference
$ErrorActionPreference = "Stop"

# Use UTF-8 for better text rendering
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# ============================================================================
# Configuration
# ============================================================================

$GithubRepo = "alessandrobrunoh/Mnemosyne"
$GithubApiUrl = "https://api.github.com/repos/$GithubRepo/releases/latest"
$RepoUrl = "https://github.com/$GithubRepo"

$UserHome = $env:USERPROFILE
if (!$UserHome) {
    $UserHome = $HOME
}

$InstallDir = "$UserHome\.mnemosyne"
$BinDir = "$InstallDir\bin"
$TempDir = Join-Path $env:TEMP "mnemosyne-install"

# Required binaries to extract
$RequiredBinaries = @("mnem.exe", "mnem-daemon.exe")

# ============================================================================
# Helper Functions
# ============================================================================

function Write-ColorOutput {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Message,

        [Parameter(Mandatory = $false)]
        [ConsoleColor]$Color = "White"
    )

    Write-Host $Message -ForegroundColor $Color
}

function Show-Banner {
    Write-ColorOutput @"
  ###    ### ###   ### ######### ###    ###
  ####  #### ####  ### ######### ####  ####
  ### ## ### ### ### ### ###     ### ## ###
  ###    ### ###  #### ###       ###    ###
  ###    ### ###   ### ######### ###    ###
  ###    ### ###   ### ######### ###    ###
"@ "Cyan"

    Write-ColorOutput "--- Mnemosyne Installation for Windows ---" "Cyan"
    Write-Host ""
}

function Stop-ExistingProcesses {
    Write-ColorOutput "[*] Stopping existing Mnemosyne processes..." "Blue"

    $processes = @("mnem", "mnem-daemon")
    $stopped = $false

    foreach ($proc in $processes) {
        try {
            $instances = Get-Process -Name $proc -ErrorAction SilentlyContinue
            if ($instances) {
                Stop-Process -Name $proc -Force -ErrorAction Stop
                Write-ColorOutput "    Stopped $proc" "Gray"
                $stopped = $true
            }
        }
        catch {
            Write-ColorOutput "    Warning: Could not stop $proc" "Yellow"
        }
    }

    if ($stopped) {
        Start-Sleep -Milliseconds 500
    }

    Write-ColorOutput "[+] Process check complete" "Green"
    Write-Host ""
}

function Get-LatestRelease {
    Write-ColorOutput "[*] Fetching latest release from GitHub..." "Blue"

    try {
        $response = Invoke-RestMethod -Uri $GithubApiUrl -Method Get -ErrorAction Stop
        $version = $response.tag_name

        if (!$version) {
            throw "Could not find tag_name in GitHub response"
        }

        Write-ColorOutput "[+] Latest version: $version" "Green"
        return $version
    }
    catch {
        Write-ColorOutput "[!] Error: Failed to fetch latest release" "Red"
        Write-ColorOutput "    Please check your internet connection" "Gray"
        Write-ColorOutput "    Repository: $RepoUrl" "Gray"
        throw
    }
}

function Download-Archive {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Version
    )

    Write-ColorOutput "[*] Downloading Windows archive..." "Blue"

    # Clean temp directory
    if (Test-Path $TempDir) {
        Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

    $zipFileName = "mnem-windows-x86_64.zip"
    $downloadUrl = "$RepoUrl/releases/download/$Version/$zipFileName"
    $zipPath = Join-Path $TempDir $zipFileName

    try {
        Write-ColorOutput "    From: $downloadUrl" "DarkGray"

        # Use Invoke-WebRequest with progress tracking
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($downloadUrl, $zipPath)

        if (!(Test-Path $zipPath)) {
            throw "Download failed - file not found"
        }

        $fileSize = (Get-Item $zipPath).Length / 1KB
        Write-ColorOutput "[+] Downloaded: $([math]::Round($fileSize, 2)) KB" "Green"

        return $zipPath
    }
    catch {
        Write-ColorOutput "[!] Error: Download failed" "Red"
        Write-ColorOutput "    URL: $downloadUrl" "Gray"
        Write-ColorOutput "    Please verify the release exists at GitHub" "Gray"
        throw
    }
}

function Expand-ArchiveFiles {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ZipPath
    )

    Write-ColorOutput "[*] Extracting archive..." "Blue"

    try {
        # Create extraction directory
        $extractDir = Join-Path $TempDir "extracted"
        New-Item -ItemType Directory -Path $extractDir -Force | Out-Null

        # Use .NET to extract ZIP (more compatible than Expand-Archive)
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [System.IO.Compression.ZipFile]::ExtractToDirectory($ZipPath, $extractDir)

        Write-ColorOutput "[+] Archive extracted" "Green"

        # Find extracted binaries
        $extractedFiles = @()
        foreach ($binary in $RequiredBinaries) {
            $filePath = Join-Path $extractDir $binary
            if (Test-Path $filePath) {
                $extractedFiles += $filePath
                Write-ColorOutput "    Found: $binary" "DarkGray"
            }
        }

        if ($extractedFiles.Count -lt $RequiredBinaries.Count) {
            throw "Missing required binaries in archive"
        }

        return $extractedFiles
    }
    catch {
        Write-ColorOutput "[!] Error: Failed to extract archive" "Red"
        Write-ColorOutput "    The archive may be corrupted or incomplete" "Gray"
        throw
    }
}

function Install-Binaries {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$SourceFiles
    )

    Write-ColorOutput "[*] Installing binaries..." "Blue"

    # Create bin directory
    if (!(Test-Path $BinDir)) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
        Write-ColorOutput "    Created directory: $BinDir" "DarkGray"
    }

    try {
        foreach ($sourceFile in $SourceFiles) {
            $fileName = Split-Path $sourceFile -Leaf
            $destPath = Join-Path $BinDir $fileName

            Copy-Item -Path $sourceFile -Destination $destPath -Force
            Write-ColorOutput "    Installed: $fileName" "DarkGray"
        }

        Write-ColorOutput "[+] Binaries installed to: $BinDir" "Green"
    }
    catch {
        Write-ColorOutput "[!] Error: Failed to copy binaries" "Red"
        Write-ColorOutput "    Target: $BinDir" "Gray"
        throw
    }
}

function Add-BinToPath {
    Write-ColorOutput "[*] Configuring PATH..." "Blue"

    try {
        $pathEnv = [Environment]::GetEnvironmentVariable("Path", "User")
        $pathEntries = $pathEnv -split ';'

        $binInPath = $false
        $normalizedBinDir = $BinDir.TrimEnd('\')

        foreach ($entry in $pathEntries) {
            $normalizedEntry = $entry.TrimEnd('\')
            if ($normalizedEntry -eq $normalizedBinDir) {
                $binInPath = $true
                break
            }
        }

        if ($binInPath) {
            Write-ColorOutput "[+] Directory already in PATH" "Green"
        }
        else {
            $newPath = "$BinDir;$pathEnv"
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")

            # Update current session
            $env:Path = "$BinDir;$env:Path"

            Write-ColorOutput "[+] Added to user PATH" "Green"
        }
    }
    catch {
        Write-ColorOutput "[!] Warning: Could not update PATH" "Yellow"
        Write-ColorOutput "    You may need to add $BinDir to PATH manually" "Gray"
    }

    Write-Host ""
}

function Show-Success {
    Write-ColorOutput "================================================" "Cyan"
    Write-ColorOutput "Installation completed successfully!" "Green"
    Write-ColorOutput "================================================" "Cyan"
    Write-Host ""
    Write-ColorOutput "IMPORTANT: Restart your terminal for PATH changes to take effect" "Yellow"
    Write-Host ""
    Write-ColorOutput "Quick Start:" "Cyan"
    Write-ColorOutput "  mnem on           Start the daemon" "White"
    Write-ColorOutput "  mnem track        Track current directory" "White"
    Write-ColorOutput "  mnem h            View history" "White"
    Write-Host ""
    Write-ColorOutput "Documentation: $RepoUrl" "Gray"
    Write-ColorOutput "To uninstall: Remove the $InstallDir folder" "Gray"
}

function Clean-TempFiles {
    Write-ColorOutput "[*] Cleaning temporary files..." "Blue"

    try {
        if (Test-Path $TempDir) {
            Remove-Item -Path $TempDir -Recurse -Force -ErrorAction Stop
        }
    }
    catch {
        Write-ColorOutput "    Warning: Could not remove temporary files" "Yellow"
    }
}

# ============================================================================
# Main
# ============================================================================

try {
    # Display banner
    Show-Banner

    # Stop existing processes
    Stop-ExistingProcesses

    # Get latest version
    $version = Get-LatestRelease

    # Download archive
    $zipPath = Download-Archive -Version $version

    # Extract and install
    $extractedFiles = Expand-ArchiveFiles -ZipPath $zipPath
    Install-Binaries -SourceFiles $extractedFiles

    # Update PATH
    Add-BinToPath

    # Clean up
    Clean-TempFiles

    # Show success message
    Show-Success

    exit 0
}
catch {
    Write-Host ""
    Write-ColorOutput "================================================" "Red"
    Write-ColorOutput "Installation failed!" "Red"
    Write-ColorOutput "================================================" "Red"
    Write-Host ""
    Write-ColorOutput "Error details:" "Red"
    Write-ColorOutput "    $_" "Gray"
    Write-Host ""
    Write-ColorOutput "For help, please visit: $RepoUrl/issues" "Gray"

    # Clean up on failure
    try {
        if (Test-Path $TempDir) {
            Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    catch {
        # Ignore cleanup errors
    }

    exit 1
}
