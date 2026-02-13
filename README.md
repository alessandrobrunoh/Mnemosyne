<div align="center">

# Mnemosyne

### Local History for Developers

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Never lose code again.** Automatic snapshots on every save, semantic code understanding, and intelligent restore—all local, zero dependencies.

</div>

---

## Why Mnemosyne?

- **Git only tracks commits** — Mnemosyne captures every save, every refactor, every experiment
- **Instant restore** — recover any previous version in milliseconds  
- **Per-project storage** — each project has its own `.mnemosyne/` folder (portable!)
- **Semantic understanding** — Tree-sitter powered AST parsing

---

## Quick Start

```bash
# 1. Start the daemon
mnem on

# 2. Track your project
cd /path/to/project
mnem track

# 3. View history
mnem h

# 4. Restore files
mnem r                    # Interactive restore
mnem r --version 5       # Restore to version 5
```

---

## Installation

### Windows
```powershell
irm https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.ps1 | iex
```

### macOS / Linux
```bash
curl -fsSL https://raw.githubusercontent.com/alessandrobrunoh/Mnemosyne/main/scripts/install.sh | bash
```

### From Source
```bash
git clone https://github.com/alessandrobrunoh/Mnemosyne.git
cd Mnemosyne
cargo build --release -p mnem-cli -p mnem-daemon
# Add binaries to your PATH
```

---

## Commands

| Command | Description |
|--------|-------------|
| `mnem on` | Start the daemon |
| `mnem off` | Stop the daemon |
| `mnem status` | Show daemon status & stats |
| `mnem track` | Track current directory |
| `mnem track --list` | List tracked projects |
| `mnem h` | View file history |
| `mnem h --branch main` | Filter by branch |
| `mnem h --limit 20` | Limit results |
| `mnem s <query>` | Search in history |
| `mnem r` | Restore files (interactive) |
| `mnem r --version 5` | Restore to version 5 |
| `mnem info` | Project statistics |
| `mnem gc` | Garbage collection |
| `mnem config` | Manage configuration |

---

## How It Works

Each project stores its data locally:

```
project/
├── .mnemosyne/          # Local storage (portable!)
│   ├── tracked          # Project ID
│   ├── db/             # SQLite database
│   └── cas/            # Content-addressable storage
└── src/                # Your code
```

**Benefits:**
- Portable — copy `.mnemosyne/` to move history
- Delete `.mnemosyne/` to remove all history
- Works offline — no cloud required

---

## Configuration

Create `.mnemignore` in your project root:

```
target/
node_modules/
*.log
```

Global config: `~/.mnemosyne/config.toml`

---

## Features

- **Auto snapshots** — every file save is captured
- **Branch tracking** — history organized by Git branches  
- **Deduplication** — BLAKE3 + Zstd compression
- **Hunk restore** — restore specific code blocks
- **Search** — full-text search across history

---

## License

MIT — See [LICENSE](LICENSE)
