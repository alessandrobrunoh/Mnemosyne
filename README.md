<div align="center">

# Mnemosyne

### Intelligent Local File History for Developers

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Architecture](https://img.shields.io/badge/architecture-event--driven-blue.svg)]()
[![Status](https://img.shields.io/badge/status-beta-yellow.svg)]()

**Never lose code again. A semantic-aware time machine that captures every file change with AST-level understanding.**

Mnemosyne is a background daemon that automatically snapshots code changes, understands language syntax through Tree-sitter, and enables intelligent restore operations—all running locally with zero external dependencies.

[Features](#features) • [Quick Start](#quick-start) • [Installation](#installation) • [Documentation](#usage) • [Architecture](#architecture)

</div>

---

## Overview

Mnemosyne solves a critical gap in the developer workflow: **the space between commits**. While Git tracks intentional checkpoints, Mnemosyne automatically captures every intermediate state—every save, every refactoring, every experiment.

### The Problem

- You refactor a complex function, then realize the original approach was better
- You want to see exactly what changed in your workflow, not what you committed
- You need to understand how a piece of code evolved over weeks of development
- You accidentally overwrite good code and need fine-grained recovery

### The Solution

Mnemosyne runs as a lightweight daemon, silently recording every file change with:

- **Automatic snapshots** on every save (zero configuration)
- **Content-addressable storage** with global deduplication (10-100x compression)
- **Semantic code understanding** via Tree-sitter AST parsing
- **Intelligent restoration** at hunk, file, or project level
- **Unified timeline** across Git branches and sessions

---

## Features

### Core Capabilities

#### 🔄 Automatic Snapshots
- Captures every file save instantly
- Works transparently in the background
- Zero-configuration file watching
- Supports all text file types

#### 💾 Efficient Storage
- **BLAKE3 cryptographic hashing** for content addressing
- **Global deduplication** across all projects
- **Zstd compression** with automatic compression detection
- **Typical compression ratio**: 10-100x reduction vs. full copies
- **Space-aware**: Automatic cleanup with configurable retention

#### 🌳 Branch Tracking & Isolation
- Automatic Git branch detection and tracking
- Filter history by branch, date, or session
- Unified timeline view across branches
- Session-based grouping (Morning, Afternoon, Evening, Night)

#### 🎯 Intelligent Restoration
- **Hunk-level restore**: Select and restore specific code blocks
- **Diff preview**: Visualize changes before restoration
- **Checkpoint & restore**: Recover entire project state
- **Safe operations**: Changes always show preview before application

#### 🔍 Powerful Search & Query
- Full-text search across entire history
- Filter by time range, file pattern, or branch
- Syntax-highlighted diff viewing
- Fast content-based lookup

#### 🌐 Multi-Language Support
- **Out-of-the-box support**: Rust, Python, TypeScript, JavaScript, Go, Java, C, C++, C#, Ruby, PHP, JSON, HTML, CSS, Markdown
- **Tree-sitter powered**: Accurate AST parsing for semantic operations
- **Extensible**: Easy to add more languages

#### ⚡ Performance Optimized
- **FastCDC chunking**: Efficient incremental change detection
- **Power awareness**: Reduces activity on battery power
- **Adaptive polling**: Smart activity detection
- **Background operation**: Minimal CPU and I/O impact

#### 🎨 Terminal User Interface
- Intuitive keyboard-driven navigation
- Multiple color themes
- Real-time file monitoring
- Session and hunk navigation
- Responsive and lightweight

---

## Quick Start

### 1. Start the Daemon

```bash
mnem start
```

The daemon initializes and begins monitoring file changes.

### 2. Start Tracking a Project

```bash
cd /path/to/your/project
mnem watch
```

Every file save is now automatically captured.

### 3. Explore Your History

```bash
mnem tui
```

Open the interactive terminal UI to browse and restore changes.

---

## Installation

### 🚀 Quick Install (Windows)

Open PowerShell and run this command to install Mnemosyne automatically:

```powershell
powershell -ExecutionPolicy ByPass -Command "iex (iwr 'https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.ps1').Content"
```

*This script will download the latest binaries (if available), install them to `%USERPROFILE%\.mnemosyne\bin`, and update your PATH automatically.*

### 🚀 Quick Install (macOS/Linux)

Open your terminal and run this command to install Mnemosyne automatically:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.sh)"
```

*This script will check for prerequisites, install pre-compiled binaries (or compile from source), and update your PATH automatically.*

> **Note:** After installation, restart your terminal or run `source ~/.zshrc` (or your shell equivalent).

### 🛠️ Manual Installation (from source)

If you have Rust installed and want to compile manually:

```bash
# Clone the repository
git clone https://github.com/alessandrobrunoh/Mnemosyne.git
cd Mnemosyne

# Install components
cargo install --path apps/mnem-cli
cargo install --path apps/mnem-daemon
```

### Prerequisites

- **macOS 10.15+**, **Linux**, or **Windows 10/11**
- **Git**
- **Rust 1.75+** (only if building from source)
- **C++ Build Tools** (required for tree-sitter and SQLite compilation)


### Systemd Auto-Start (Linux)

Create `~/.config/systemd/user/mnem-daemon.service`:

```ini
[Unit]
Description=Mnemosyne File History Daemon
After=network.target

[Service]
Type=simple
ExecStart=%h/.mnemosyne/bin/mnem-daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user daemon-reload
systemctl --user enable mnem-daemon
systemctl --user start mnem-daemon
```

### LaunchAgent Auto-Start (macOS)

Create `~/Library/LaunchAgents/com.mnemosyne.daemon.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mnemosyne.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOUR_USERNAME/.mnemosyne/bin/mnem-daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load:

```bash
launchctl load ~/Library/LaunchAgents/com.mnemosyne.daemon.plist
launchctl start com.mnemosyne.daemon
```

---

## Usage

### Command-Line Interface

#### View File History
```bash
mnem log src/main.rs
mnem log src/main.rs --limit 50
```

#### Search Across History
```bash
mnem search "function_name"
mnem search "TODO" --pattern "*.rs" --after "2024-01-01"
```

#### Restore from History
```bash
# Interactive hunk selection
mnem restore src/main.rs --interactive

# Restore specific save
mnem restore src/main.rs --save-id <hash>

# Restore entire project state
mnem restore --checkpoint <timestamp>
```

#### Project Information
```bash
mnem info                    # Project statistics
mnem list                    # All tracked projects
mnem activity                # Recent activity summary
```

#### Maintenance
```bash
mnem gc                      # Garbage collection (remove unreferenced data)
mnem status                  # Daemon status and health
mnem stop                    # Stop the daemon
```

### Terminal UI

Launch the interactive browser:

```bash
mnem tui
```

#### Navigation Keybindings

**Global**
- `q` / `Ctrl+C` - Quit
- `Tab` - Switch between panels
- `↑↓` / `j↓k` - Navigate
- `Enter` - Select item
- `?` - Help

**File History**
- `n` / `j` - Next save
- `p` / `k` - Previous save
- `s` - Jump to session
- `/` - Search contents
- `d` - View diff

**Restore Preview**
- `Space` - Toggle hunk selection
- `a` - Select all hunks
- `r` - Restore selected
- `Esc` - Cancel

**Session Navigation**
- `M` - Morning session
- `A` - Afternoon session
- `E` - Evening session
- `N` - Night session

---

## Configuration

Mnemosyne reads configuration from:
- **Linux/macOS**: `~/.mnemosyne/config.toml`
- **Windows**: `%USERPROFILE%\.mnemosyne\config.toml`

### Default Configuration

```toml
# Data retention period (days)
retention_days = 365

# Enable/disable compression (recommended: true)
compression_enabled = true

# Respect .gitignore files
use_gitignore = true

# Respect .mnemosyneignore files
use_mnemosyneignore = true

# Terminal UI theme (0 = default, 1 = dark, 2 = light)
theme_index = 0

# Maximum file size to track (MB)
max_file_size_mb = 10

# Battery optimization (reduce activity when on battery)
power_optimization = true

# File monitoring interval (ms)
monitor_interval_ms = 1000

# Track and record Git branch information
track_git_branch = true

# Number of parallel workers for hashing
worker_threads = 4
```

### Project-Specific Ignore Rules

Create `.mnemignore` in your project root (syntax similar to `.gitignore`):

```
# Ignore patterns
target/
*.log
node_modules/
.DS_Store
__pycache__/

# Explicitly include certain files
!src/main.rs
```

---

## Architecture

Mnemosyne is structured as a modular Rust workspace with event-driven architecture:

```
┌──────────────────────────────────────────────────────────────┐
│                         User Interface                        │
│  ┌──────────────────┐              ┌──────────────────┐     │
│  │   mnem-cli       │              │   mnem-tui       │     │
│  │  (Command-line)  │              │  (Interactive)   │     │
│  └────────┬─────────┘              └────────┬─────────┘     │
└───────────┼────────────────────────────────┼─────────────────┘
            │                                │
        ┌───▼────────────────────────────────▼────┐
        │         mnem-core (Library)             │
        │                                         │
        │  • Storage layer (BLAKE3, Zstd)       │
        │  • Data models                         │
        │  • Semantic parsing (Tree-sitter)     │
        │  • Client/IPC protocol                │
        │  • Diff & merge algorithms            │
        └────────────────┬──────────────────────┘
                         │
        ┌────────────────▼──────────────────┐
        │      mnem-daemon (Daemon)         │
        │                                   │
        │  • File system monitoring         │
        │  • Event processing loop          │
        │  • Power management               │
        │  • IPC server                     │
        │  • Storage coordination           │
        └───────────────────────────────────┘
```

### Architectural Decisions

For a deep dive into the engineering behind Mnemosyne, see the technical design docs:
- [Protocol Standardization (MNP v1.0)](docs/MNP_STANDARDIZATION.md)
- [Semantic Deltas & AST Tracking](docs/SEMANTIC_DELTAS.md)
- [High-Performance Architecture](docs/PERFORMANCE_ARCHITECTURE.md)

#### 1. Hybrid Storage Strategy (Relational + CAS)
Mnemosyne uses a dual-layer storage architecture designed for both query speed and space efficiency:
- **SQLite (Metadata & Relations)**: Handles the "Graph of Time". It manages relationships between snapshots, symbols, and deltas. By using SQLite in WAL (Write-Ahead Logging) mode, we achieve ACID compliance and crash resilience with minimal overhead.
- **Content-Addressable Storage (Blobs)**: Actual code content is stored as deduplicated chunks in the filesystem, indexed by **BLAKE3** hashes and compressed with **Zstd**. This mirrors Git's internals but is optimized for frequent, small saves.

#### 2. Semantic Deltas vs. Binary Deltas
Unlike traditional local history tools (like IntelliJ) that use binary diffs (VCDIFF) to save space, Mnemosyne implements **Semantic Deltas**:
- **AST-Aware**: By leveraging Tree-sitter, we track changes to specific logical entities (functions, classes) rather than just lines of text.
- **Structural Identity**: We use a `structural_hash` to identify code logic. If a function is renamed or moved to a different file, Mnemosyne recognizes it as the same entity, maintaining a continuous evolution timeline where others see a deletion and a new insertion.

#### 3. Protocol-First Design (MNP v1.0)
Mnemosyne is built as a **Language Server for History**. The core daemon communicates via the **Mnemosyne Protocol (MNP)**, a JSON-RPC 2.0 based standard.
- **Decoupled**: The backend is completely agnostic of the editor.
- **Extensible**: MNP v1.0 includes formal lifecycle management (`initialize`, `capabilities negotiation`) similar to LSP, allowing anyone to build new clients (VSCode, Vim, CLI tools) that instantly benefit from the semantic history engine.

### Component Responsibilities


| Crate | Role |
|-------|------|
| **mnem-core** | Core library: storage engine, models, semantic analysis, client protocol |
| **mnem-daemon** | Daemon: file monitoring, event loop, IPC server, power optimization |
| **mnem-cli** | Command-line interface: all `mnem` commands |
| **mnem-tui** | Terminal UI: interactive history browser |
| **mnem-test** | Testing utilities and integration tests |

### Data Flow

```
File Save → File Watcher → Event Queue → Daemon Processing
                                              ↓
                                        FastCDC Chunking
                                              ↓
                                        BLAKE3 Hashing
                                              ↓
                                        Deduplication
                                              ↓
                                   Zstd Compression (optional)
                                              ↓
                                         SQLite Index
                                              ↓
                                        Disk Storage
```

---

## Comparison with Alternatives

| Aspect | Git | IDE History | Mnemosyne |
|--------|-----|-------------|-----------|
| **Granularity** | Manual commits | Coarse (minutes) | Per-save (seconds) |
| **Storage** | Full file copies | Incremental | Content-addressed + deduplication |
| **Code understanding** | Text-based | Syntax highlighting | AST-aware (Tree-sitter) |
| **Query capability** | Commit messages | File/date | Function-level semantic |
| **Restore granularity** | Files | Full files | Individual hunks |
| **Background operation** | Manual | Continuous | Continuous, optimized |
| **Setup complexity** | Simple | Built-in | Minimal (single daemon) |

---

## Roadmap

### Version 1.0 (Current - The "Standard" Milestone)
- ✅ **Mnemosyne Protocol (MNP) v1.0**: Standardized JSON-RPC lifecycle.
- ✅ **Semantic Deltas**: AST-based change tracking (Added, Modified, Renamed, Deleted).
- ✅ **Symbol Timeline**: Evolutionary history of logical entities across renames.
- ✅ **Global Deduplication**: Content-addressed storage with BLAKE3.
- ✅ **Terminal UI & CLI**: Full-featured interactive and command-line tools.

### Version 1.1 (Q2 2026)
- 🔄 **IDE Plugin Ecosystem**: Official extensions for Zed, VS Code, and Neovim.
- 🔄 **JSON Schema**: Formal schemas for all protocol messages.
- 🔄 **Advanced Rename Detection**: Fuzzy matching for complex refactors.

### Version 1.2 (Q3 2026)
- 📅 **Optional encrypted cloud sync**
- 📅 **Collaborative history sharing**
- 📅 **Web Dashboard**


### Future
- 🔮 AI-powered change summarization
- 🔮 Anomaly detection ("suspicious commits")
- 🔮 CI/CD pipeline integration

---

## Performance Characteristics

### Storage Efficiency
- **Typical compression**: 10-100x vs. uncompressed file copies
- **Deduplication**: Identical content blocks deduplicated globally
- **Example**: 1 year of daily work on a 10MB codebase typically requires 50-200MB

### CPU & Memory
- **Daemon overhead**: < 1% CPU at idle, ~5% during active save operations
- **Memory usage**: ~50-100MB resident
- **Latency**: Snapshot captured within 100-500ms after file save

### Scalability
- **Project size**: Tested with codebases 100MB+
- **History depth**: Handles millions of snapshots
- **File count**: Efficiently handles 10,000+ files per project

---

## Contributing

Contributions are welcome. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/alessandrobrunoh/Mnemosyne.git
cd Mnemosyne

# Build and run tests
cargo test --workspace

# Run with debug logging
RUST_LOG=debug cargo run -p mnem-daemon
```

### Code Standards

- Follow Rust API Guidelines
- Use `cargo fmt` for formatting
- Pass `cargo clippy` linting
- Write tests for new features
- Document public APIs

### Areas for Contribution

- Additional language support (expand Tree-sitter grammars)
- Performance optimizations
- Platform support (Windows, BSD)
- IDE integrations
- Documentation and guides
- Test coverage expansion

---

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- **Tree-sitter** - Parser generator technology
- **ratatui** - Terminal UI framework
- **BLAKE3** - Fast cryptographic hashing
- **FastCDC** - Content-aware chunking algorithm
- **notify** - File system event monitoring
- **Zstandard** - Efficient compression algorithm

---

## Support

- **Issues**: [GitHub Issues](https://github.com/alessandrobrunoh/Mnemosyne/issues)
- **Discussions**: [GitHub Discussions](https://github.com/alessandrobrunoh/Mnemosyne/discussions)
- **Documentation**: [Mnemosyne Docs](https://docs.mnemosyne.dev) (coming soon)

---

<div align="center">

**Developed by the Mnemosyne Team**

*Capture every change. Understand your code's evolution.*

</div>
