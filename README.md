# Mnemosyne (MNP)

**Semantic Code Archaeology & High-Performance Memory for Your Codebase.**

Mnemosyne is a high-performance, local-first platform designed to provide an eternal memory for your development workflow. By leveraging Tree-sitter for semantic understanding and a content-addressable storage (CAS) architecture, Mnemosyne tracks the evolution of your code not just as text diffs, but as logical mutations of symbols, functions, and structures.

[![CI](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/ci.yml/badge.svg)](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/ci.yml)
[![Build](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/build.yml/badge.svg)](https://github.com/alessandrobrunoh/Mnemosyne/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

---

## Core Pillars

### 1. Semantic Identity over Textual Diffs
Traditional version control tracks lines. Mnemosyne tracks **logic**. Using Tree-sitter ASTs and `structural_hash` algorithms, it maintains continuity across renames, refactors, and movements. It understands when a function has moved or when a struct has evolved, providing a true "archaeological" record of your symbols.

### 2. High-Performance Engineering
Built in Rust with a focus on zero-copy operations and granular concurrency:
*   **Zero-Copy Data Handling**: Utilizes `bytes::Bytes` and `mmap` to minimize CPU overhead.
*   **Lock-Free Concurrency**: Uses concurrent collections (`DashMap`) and atomics to ensure the daemon never blocks your flow.
*   **Storage Architecture**: Powered by `redb` as a high-performance B-tree index and a custom CAS layer for efficient deduplication.

### 3. Model Context Protocol (MCP) Integration
Mnemosyne isn't just for humans. It acts as a powerful context layer for LLMs and AI Agents via the **Mnemosyne Protocol (MNP)**. It allows agents to:
*   Retrieve semantic deltas instead of raw files (saving tokens).
*   Access the historical evolution of specific symbols.
*   Understand the "why" behind architectural changes through indexed metadata.

---

## Workspace Architecture

The project is structured as a modular workspace for maximum reusability:

| Component | Description |
|-----------|-------------|
| `mnem-daemon` | The background engine managing file watching, AST parsing, and storage. |
| `mnem-cli` | A polished CLI for project management and quick history access. |
| `mnem-tui` | A feature-rich terminal interface for visual history exploration. |
| `mnem-core` | Shared logic: IPC, MNP protocol, CAS storage, and database schemas. |
| `mnem-mcp` | Model Context Protocol server for integrating with LLMs (Claude, etc.). |
| `mnem-lsp` | Language Server Protocol bridge for IDE-native features. |
| `mnem-zed` | Native extension for the Zed editor. |

---

## Getting Started

### Installation

#### One-line Install (Recommended)
**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.ps1 | iex
```

#### From Source
```bash
git clone https://github.com/alessandrobrunoh/Mnemosyne.git
cd Mnemosyne
cargo build --release
```

### Quick Start
```bash
# Start the background daemon
mnem daemon start

# Initialize tracking in your current project
mnem track

# Open the interactive TUI to explore history
mnem ui

# Search for a specific symbol's history
mnem search --symbol "process_data"
```

---

## Configuration

Mnemosyne respects your project boundaries through `.mnemignore` files and a global `config.toml`.

**Global Config (`~/.mnemosyne/config.toml`):**
```toml
[storage]
retention_days = 30
compression_enabled = true
max_file_size_mb = 10

[editor]
ide = "Zed"
```

---

## License

This project is licensed under the **Apache License 2.0**. See the [LICENSE](LICENSE) file for details.
