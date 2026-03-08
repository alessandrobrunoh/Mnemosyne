<div align="center">

# Mnemosyne
**Never lose code again.** Sync your history across your favorite IDEs. Local snapshots, semantic understanding, and instant restore—all offline.

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](LICENSE)

</div>

---

## Why Mnemosyne?

...

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

### From Source
```bash
git clone https://github.com/alessandrobrunoh/Mnemosyne.git
cd Mnemosyne/mnemosyne
cargo build --release -p mnem-cli -p mnem-daemon
# Copy binaries to your PATH
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

## Integrations

- **CLI** — Full-featured command line
- **VSCode** — Coming soon
- **Zed** — Coming soon
- **JetBrains** - Coming soon

---

## License

MIT — See [LICENSE](LICENSE)
