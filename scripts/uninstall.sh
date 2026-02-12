#!/bin/bash

# Mnemosyne Uninstallation Script for macOS/Linux
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

INSTALL_DIR="$HOME/.mnemosyne"
BIN_DIR="$INSTALL_DIR/bin"

echo -e "${RED}--- Mnemosyne Uninstallation ---${NC}"

# 1. Stop processes
echo -e "${BLUE}[*] Stopping Mnemosyne processes...${NC}"
pkill -9 mnem 2>/dev/null || true
pkill -9 mnem-daemon 2>/dev/null || true
sleep 1

# 2. Remove files
if [ -d "$INSTALL_DIR" ]; then
    echo -e "${BLUE}[*] Removing $INSTALL_DIR...${NC}"
    rm -rf "$INSTALL_DIR"
    echo -e "${GREEN}✓ Removed files.${NC}"
else
    echo -e "${YELLOW}[!] $INSTALL_DIR not found.${NC}"
fi

# 3. Clean up PATH (optional, but good practice)
SHELL_NAME="$(basename "$SHELL")"
case "$SHELL_NAME" in
    zsh) CONFIG_FILE="$HOME/.zshrc" ;;
    bash) [ -f "$HOME/.bashrc" ] && CONFIG_FILE="$HOME/.bashrc" || CONFIG_FILE="$HOME/.bash_profile" ;;
    *) CONFIG_FILE="" ;;
esac

if [ -n "$CONFIG_FILE" ] && [ -f "$CONFIG_FILE" ]; then
    if grep -q "Mnemosyne bin" "$CONFIG_FILE"; then
        echo -e "${BLUE}[*] Cleaning up $CONFIG_FILE...${NC}"
        # Remove the lines added by the installer
        # This is a bit tricky with sed, we'll just advise the user or do a simple removal
        sed -i.bak '/# Mnemosyne bin/d' "$CONFIG_FILE"
        sed -i.bak '/export PATH="\$PATH:'"$(echo $BIN_DIR | sed 's/\//\//g')"'"/d' "$CONFIG_FILE"
        rm -f "${CONFIG_FILE}.bak"
        echo -e "${GREEN}✓ Cleaned up $CONFIG_FILE.${NC}"
    fi
fi

# 4. Remove LaunchAgent (macOS)
if [ "$(uname)" = "Darwin" ]; then
    PLIST="$HOME/Library/LaunchAgents/com.mnemosyne.daemon.plist"
    if [ -f "$PLIST" ]; then
        echo -e "${BLUE}[*] Removing LaunchAgent...${NC}"
        launchctl unload "$PLIST" 2>/dev/null || true
        rm -f "$PLIST"
        echo -e "${GREEN}✓ Removed LaunchAgent.${NC}"
    fi
fi

echo ""
echo -e "${GREEN}Mnemosyne has been uninstalled.${NC}"
echo -e "${YELLOW}Note: You may need to restart your terminal session for the PATH changes to take effect.${NC}"
