#!/bin/bash
# Installation script for Mnemosyne LSP Server
# This script builds and installs mnem-lsp to your system

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect operating system
detect_os() {
    case "$(uname -s)" in
        Darwin*)    echo "macos";;
        Linux*)     echo "linux";;
        CYGWIN*)    echo "windows";;
        MINGW*)     echo "windows";;
        *)          echo "unknown";;
    esac
}

# Check if Rust is installed
check_rust() {
    log_info "Checking Rust installation..."

    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi

    local rust_version=$(rustc --version)
    log_success "Found $rust_version"
}

# Check if Mnemosyne daemon is installed
check_mnemosyne() {
    log_info "Checking Mnemosyne installation..."

    if ! command -v mnemd &> /dev/null; then
        log_warning "Mnemosyne daemon (mnemd) not found in PATH"
        log_info "You can still install mnem-lsp, but ensure mnemd is available when using the LSP"
    else
        log_success "Found Mnemosyne daemon: $(mnemd --version 2>&1 || echo 'version unknown')"
    fi
}

# Build the LSP server
build_lsp() {
    log_info "Building mnem-lsp in release mode..."

    # Get the directory where this script is located
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

    # Build from project root
    cd "$PROJECT_ROOT"

    if cargo build --release -p mnem-lsp; then
        log_success "Build completed successfully"
    else
        log_error "Build failed. Please check the error messages above."
        exit 1
    fi

    cd - > /dev/null
}

# Install the binary
install_binary() {
    local os=$(detect_os)
    local binary_name="mnem-lsp"
    local install_dir="/usr/local/bin"

    log_info "Installing mnem-lsp binary..."

    # Get the directory where this script is located
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
    local binary_path="$PROJECT_ROOT/target/release/$binary_name"

    # Check if binary exists
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found at $binary_path"
        log_info "Did the build succeed?"
        exit 1
    fi

    # Determine install directory based on OS and permissions
    if [ "$os" = "macos" ]; then
        # On macOS, prefer /usr/local/bin or /opt/homebrew/bin
        if [ -w "/usr/local/bin" ]; then
            install_dir="/usr/local/bin"
        elif [ -d "/opt/homebrew/bin" ] && [ -w "/opt/homebrew/bin" ]; then
            install_dir="/opt/homebrew/bin"
        fi
    fi

    # Check if we need sudo
    if [ -w "$install_dir" ]; then
        cp "$binary_path" "$install_dir/$binary_name"
        chmod +x "$install_dir/$binary_name"
        log_success "Installed to $install_dir/$binary_name"
    else
        log_info "Administrator privileges required to install to $install_dir"
        sudo cp "$binary_path" "$install_dir/$binary_name"
        sudo chmod +x "$install_dir/$binary_name"
        log_success "Installed to $install_dir/$binary_name"
    fi

    # Verify installation
    if command -v mnem-lsp &> /dev/null; then
        log_success "mnem-lsp is now available in your PATH"
    else
        log_warning "mnem-lsp was installed but is not in your PATH"
        log_info "Add $install_dir to your PATH or restart your terminal"
    fi
}

# Print Zed configuration instructions
print_zed_config() {
    cat <<EOF

${BLUE}=======================================${NC}
${GREEN}Installation Complete!${NC}
${BLUE}=======================================${NC}

Next steps to configure Zed IDE:

1. Open Zed settings (Cmd+, on macOS, Ctrl+, on Linux)

2. Add the following to your settings.json:

${YELLOW}---${NC}
{
  "languages": {
    "Rust": {
      "language_servers": ["rust-analyzer", "mnem-lsp"]
    }
  },
  "lsp": {
    "mnem-lsp": {
      "binary": {
        "path": "$(command -v mnem-lsp || echo '/usr/local/bin/mnem-lsp')",
        "arguments": []
      }
    }
  }
}
${YELLOW}---${NC}

3. For other languages, replace "Rust" with:
   - Python, TypeScript, JavaScript, Go, Java, C, C++, etc.

4. Start Mnemosyne daemon and watch your project:
   ${BLUE}mnem start${NC}
   ${BLUE}cd /your/project${NC}
   ${BLUE}mnem watch${NC}

5. Restart Zed to pick up the new LSP server

${BLUE}=======================================${NC}
${GREEN}Usage:${NC}
${BLUE}=======================================${NC}

- ${BLUE}Hover${NC} over any symbol to see version count and last modification
- ${BLUE}Cmd+Click${NC} (macOS) or ${BLUE}Ctrl+Click${NC} (Linux) to navigate historical versions
- ${BLUE}F12${NC} also works for goto definition

${BLUE}=======================================${NC}
${GREEN}Documentation:${NC}
${BLUE}=======================================${NC}

Full documentation: Gemini/crates/mnem-lsp/README.md
Example configurations: Gemini/crates/mnem-lsp/examples/zed-settings.json

${BLUE}=======================================${NC}
${GREEN}Troubleshooting:${NC}
${BLUE}=======================================${NC}

If LSP doesn't work:
1. Check daemon status: ${BLUE}mnem status${NC}
2. Verify project is watched: ${BLUE}mnem watch${NC}
3. Check Zed logs: Cmd+Shift+P -> "zed: open log"
4. Run with debug: ${BLUE}RUST_LOG=debug mnem-lsp${NC}

EOF
}

# Uninstall function
uninstall() {
    log_info "Uninstalling mnem-lsp..."

    local binary_path=$(command -v mnem-lsp)

    if [ -z "$binary_path" ]; then
        log_warning "mnem-lsp is not installed"
        exit 0
    fi

    if [ -w "$binary_path" ]; then
        rm "$binary_path"
        log_success "Uninstalled mnem-lsp"
    else
        sudo rm "$binary_path"
        log_success "Uninstalled mnem-lsp"
    fi
}

# Main installation process
main() {
    echo -e "${BLUE}=======================================${NC}"
    echo -e "${GREEN}Mnemosyne LSP Server Installer${NC}"
    echo -e "${BLUE}=======================================${NC}"
    echo

    # Parse arguments
    case "${1:-install}" in
        uninstall)
            uninstall
            exit 0
            ;;
        install)
            ;;
        *)
            log_error "Unknown command: $1"
            echo "Usage: $0 [install|uninstall]"
            exit 1
            ;;
    esac

    # Run installation steps
    check_rust
    check_mnemosyne
    build_lsp
    install_binary
    print_zed_config

    log_success "Installation completed successfully!"
}

# Run main with all arguments
main "$@"
