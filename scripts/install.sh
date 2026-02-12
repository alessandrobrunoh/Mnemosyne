#!/bin/bash

# Mnemosyne Installation Script for macOS/Linux
# This script downloads pre-compiled binaries from GitHub releases

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="$HOME/.mnemosyne"
BIN_DIR="$INSTALL_DIR/bin"
REPO="alessandrobrunoh/Mnemosyne"

echo -e "${CYAN}--- Mnemosyne Quick Installation ---${NC}"
echo ""

# 1. Detect platform and architecture
echo -e "${BLUE}[1/4] Detecting your platform...${NC}"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Darwin)
        PLATFORM="macos"
        ;;
    Linux)
        PLATFORM="linux"
        ;;
    *)
        echo -e "${RED}❌ Unsupported OS: $OS${NC}"
        echo "This script supports macOS and Linux only."
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH_SUFFIX="x86_64"
        ;;
    aarch64|arm64)
        if [ "$PLATFORM" = "macos" ]; then
            ARCH_SUFFIX="arm64"
        else
            echo -e "${RED}❌ ARM64 not supported on Linux yet${NC}"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}✓${NC} Detected: $PLATFORM $ARCH_SUFFIX"

# 2. Get latest release version
echo -e "${BLUE}[2/4] Checking latest release...${NC}"

LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"

if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
    echo -e "${RED}❌ Neither curl nor wget found${NC}"
    echo "Please install curl or wget first."
    exit 1
fi

if command -v curl &> /dev/null; then
    RELEASE_INFO=$(curl -s "$LATEST_URL")
    VERSION=$(echo "$RELEASE_INFO" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
else
    RELEASE_INFO=$(wget -qO- "$LATEST_URL")
    VERSION=$(echo "$RELEASE_INFO" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
fi

if [ -z "$VERSION" ]; then
    echo -e "${RED}❌ Could not determine latest version${NC}"
    echo "Please check: https://github.com/${REPO}/releases"
    exit 1
fi

echo -e "${GREEN}✓${NC} Latest version: $VERSION"

# 3. Download binaries
echo -e "${BLUE}[3/4] Downloading binaries...${NC}"

# Stop any running instances
pkill -9 mnem 2>/dev/null || true
pkill -9 mnemd 2>/dev/null || true
sleep 1

# Create install directory
mkdir -p "$BIN_DIR"

# Download function
download_file() {
    local url="$1"
    local output="$2"
    local name="$3"

    echo -e "  Downloading ${name}..."

    if command -v curl &> /dev/null; then
        if ! curl -sSL -o "$output" "$url"; then
            echo -e "${RED}❌ Failed to download ${name}${NC}"
            return 1
        fi
    else
        if ! wget -q -O "$output" "$url"; then
            echo -e "${RED}❌ Failed to download ${name}${NC}"
            return 1
        fi
    fi

    return 0
}

# Base URL for downloads
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

# Download and extract binaries
CLI_ARCHIVE="mnem-${PLATFORM}-${ARCH_SUFFIX}.tar.gz"
DAEMON_ARCHIVE="mnem-daemon-${PLATFORM}-${ARCH_SUFFIX}.tar.gz"

CLI_URL="${BASE_URL}/${CLI_ARCHIVE}"
DAEMON_URL="${BASE_URL}/${DAEMON_ARCHIVE}"

TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

CLI_TEMP="$TEMP_DIR/$CLI_ARCHIVE"
DAEMON_TEMP="$TEMP_DIR/$DAEMON_ARCHIVE"

# Download both binaries
if ! download_file "$CLI_URL" "$CLI_TEMP" "mnem CLI"; then
    echo -e "${YELLOW}⚠️  Pre-compiled binaries not found for this platform${NC}"
    echo "You may need to compile from source:"
    echo "  git clone https://github.com/${REPO}.git"
    echo "  cd Mnemosyne"
    echo "  cargo build --release -p mnem-cli -p mnem-daemon"
    exit 1
fi

download_file "$DAEMON_URL" "$DAEMON_TEMP" "mnem daemon"

# Extract binaries
echo -e "  Extracting binaries..."
cd "$TEMP_DIR"

if ! tar -xzf "$CLI_ARCHIVE" 2>/dev/null; then
    echo -e "${RED}❌ Failed to extract ${CLI_ARCHIVE}${NC}"
    exit 1
fi

if ! tar -xzf "$DAEMON_ARCHIVE" 2>/dev/null; then
    echo -e "${RED}❌ Failed to extract ${DAEMON_ARCHIVE}${NC}"
    exit 1
fi

# Copy to install directory
cp mnem "$BIN_DIR/mnem"
cp mnem-daemon "$BIN_DIR/mnem-daemon"
chmod +x "$BIN_DIR/mnem"
chmod +x "$BIN_DIR/mnem-daemon"

echo -e "${GREEN}✓${NC} Binaries installed in $BIN_DIR"

# 4. Update PATH
echo -e "${BLUE}[4/4] Configuring PATH...${NC}"

# Detect shell and config file
SHELL_NAME="$(basename "$SHELL")"
CONFIG_FILE=""

case "$SHELL_NAME" in
    zsh)
        CONFIG_FILE="$HOME/.zshrc"
        ;;
    bash)
        if [ -f "$HOME/.bashrc" ]; then
            CONFIG_FILE="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
            CONFIG_FILE="$HOME/.bash_profile"
        else
            CONFIG_FILE="$HOME/.bash_profile"
        fi
        ;;
    *)
        echo -e "${YELLOW}⚠️  Unknown shell: $SHELL_NAME${NC}"
        echo -e "${YELLOW}Please add $BIN_DIR to your PATH manually${NC}"
        CONFIG_FILE=""
        ;;
esac

# Add to PATH if not already present
if [ -n "$CONFIG_FILE" ]; then
    if ! grep -q "Mnemosyne bin" "$CONFIG_FILE" 2>/dev/null; then
        echo "" >> "$CONFIG_FILE"
        echo "# Mnemosyne bin" >> "$CONFIG_FILE"
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$CONFIG_FILE"
        echo -e "${GREEN}✓${NC} Added to PATH in $CONFIG_FILE"
    else
        echo -e "${GREEN}✓${NC} Already in PATH ($CONFIG_FILE)"
    fi
fi

# Export for current session
export PATH="$PATH:$BIN_DIR"

# 5. Verify installation
echo ""
echo -e "${BLUE}Verifying installation...${NC}"

if [ -x "$BIN_DIR/mnem" ]; then
    echo -e "${GREEN}✓${NC} Mnemosyne installed successfully!"
else
    echo -e "${RED}❌ Installation verification failed${NC}"
    exit 1
fi

# 6. Summary
echo ""
echo -e "${CYAN}─────────────────────────────────────────────${NC}"
echo -e "${GREEN}✓ Installation complete!${NC}"
echo -e "${CYAN}─────────────────────────────────────────────${NC}"
echo ""
echo -e "${BLUE}Quick Start:${NC}"
echo "  1. Restart your terminal or run:"
echo -e "     ${YELLOW}source $CONFIG_FILE${NC}"
echo ""
echo "  2. Start the daemon:"
echo -e "     ${YELLOW}mnem start${NC}"
echo ""
echo "  3. Track a project:"
echo -e "     ${YELLOW}cd /path/to/your/project${NC}"
echo -e "     ${YELLOW}mnem watch${NC}"
echo ""
echo -e "${BLUE}For more information:${NC}"
echo "  mnem --help"
echo "  mnem tui    # Interactive terminal UI"
echo ""
