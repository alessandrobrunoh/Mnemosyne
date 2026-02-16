<div align="center">

# Mnemosyne

### Local History for Developers

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](LICENSE)

**Never lose code again.** Sync your history across your favorite IDEs. Local snapshots, semantic understanding, and instant restore—all offline.

</div>

---

## The Problem We Solve

Every developer knows this pain:

> **It's 11 PM. You've been refactoring for 3 hours. Suddenly—accident.** You press `Ctrl+Z` one too many times. The function you spent all evening building is gone. No git commit. No backup. Just... gone.

**This is why Mnemosyne exists.**

Git is great for commits—but what about everything in between? Every developer loses work between commits. Mnemosyne captures **every save**, so you can always go back.

| Before Mnemosyne | After Mnemosyne |
|-----------------|-----------------|
| Ctrl+Z forever (and then too far) | Instant restore of **any** previous save |
| "I should commit" → forget → regret | Auto-captures every save |
| "When did I delete this?" | Full searchable history |
| 50 versions = 50 full copies | 50 versions = ~1 copy (dedup) |

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
mnem r <path/to/file> --list       # See all versions
mnem r <path/to/file> 1            # Restore verison 1
```

---

## Installation

### Windows (PowerShell)
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
cd Mnemosyne/mnemosyne
cargo build --release -p mnem-cli -p mnem-daemon
# Copy binaries to your PATH
```

---

## Commands

### Daemon
| Command | Description |
|--------|-------------|
| `mnem on` | Start daemon |
| `mnem off` | Stop daemon |
| `mnem status` | Show status & stats |

### Tracking
| Command | Description |
|--------|-------------|
| `mnem track` | Track current directory |
| `mnem track --list` | List tracked projects |

### History
| Command | Description |
|--------|-------------|
| `mnem h` | View history |
| `mnem h --branch main` | Filter by branch |
| `mnem h --limit 20` | Limit results |
| `mnem h --timeline` | Timeline view |

### Search & Restore
| Command | Description |
|--------|-------------|
| `mnem s <query>` | Search in history |
| `mnem r` | Interactive restore |
| `mnem r --version 5` | Restore to version 5 |
| `mnem r --undo` | Undo last restore |

### Info & Maintenance
| Command | Description |
|--------|-------------|
| `mnem info` | Project statistics |
| `mnem gc` | Garbage collection |
| `mnem config` | Manage configuration |

---

## How It Works

### Per-Project Storage

Each project stores its data locally in `.mnemosyne/`:

```
my-project/
├── .mnemosyne/          # All data lives here!
│   ├── tracked          # Project ID
│   ├── db/             # redb (snapshots, symbols, interning)
│   └── cas/            # Content-addressable storage (unique chunks)
├── src/
│   └── main.rs
└── Cargo.toml
```

**Benefits:**
- Portable — copy project to move history
- Delete `.mnemosyne/` to remove all history  
- Works offline — no cloud required
- No global state pollution


### Semantic Understanding

Mnemosyne uses **Tree-sitter** to understand code structure:

- Tracks **functions, classes, structs** — not just lines
- Survives **renames and refactors**
- **Deduplicates** using BLAKE3 hashing
- **Compresses** with Zstd (Level 3 optimized for speed)

---

## Configuration

### Project Ignore

Create `.mnemignore` in your project root:

```
target/
node_modules/
*.log
*.tmp
build/
dist/
```

### Global Config

`~/.mnemosyne/config.toml`:

```toml
[daemon]
auto_start = true
poll_interval_ms = 500

[storage]
compression = true
deduplicate = true

[ignore]
global = ["*.log", "*.tmp"]
```

---

## Features

- **Auto Snapshots** — Every file save captured automatically
- **Branch Tracking** — History organized by Git branch
- **Semantic Deltas** — Understands code structure (functions, classes)
- **Instant Restore** — Millisecond recovery to any point
- **Full-Text Search** — Search across all history
- **10-100x Storage** — Deduplication vs full copies
- **Symbol History** — Track how functions and classes evolve
- **IDE Integration** — Open versions in your editor

---

## Use Cases

1. **Recover lost work** — "I accidentally deleted this function"
2. **See evolution** — "How did I implement this feature?"
3. **Compare approaches** — "What did I try before?"
4. **Debug regressions** — "When did this break?"
5. **Share snapshots** — Send a version to a colleague

---

## Integrations

- **VSCode** — Coming soon
- **Zed** — Built-in Mnemosyne support
- **CLI** — Full-featured command line

---

## License

APACHE 2.0 — See [LICENSE](LICENSE)
