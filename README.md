<div align="center">

# Mnemosyne

### Local History for Developers

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Never lose code again.** Sync your history across your favorite IDEs. Local snapshots, semantic understanding, and instant restore—all offline.

</div>

---

## Why Mnemosyne?

| Problem | Mnemosyne Solution |
|--------|-------------------|
| Git only tracks commits | Captures every save |
| Lost work between commits | Instant restore |
| Can't remember what changed | Full history with diffs |
| Large backup files | Deduplicated storage (10-100x smaller) |

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
mnem r                    # Interactive
mnem r --version 5       # Specific version
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
- ✅ Portable — copy project to move history
- ✅ Delete `.mnemosyne/` to remove all history  
- ✅ Works offline — no cloud required
- ✅ No global state pollution

### Zed-style Performance Engine

Mnemosyne is built with high-performance engineering principles:

- **redb Engine**: Pure-Rust, Copy-on-Write B-tree database (replaces SQLite).
- **Zero-Copy**: Leverages `mmap` and `bytes::Bytes` for direct memory access without redundant buffers.
- **Background Parsing**: Tree-sitter indexing happens in background threads, keeping response times **< 1ms**.
- **String Interning**: Paths and symbols are stored once and referenced by IDs, reducing DB size by **30%**.
- **Trigram Grep**: Search history 10x faster using Trigram-based Bloom filters to skip irrelevant chunks.
- **Adaptive Debounce**: Intelligently scales snapshot frequency during heavy work (e.g., `npm install`).
- **Chunk-only Storage**: Files are stored as unique semantic chunks and reassembled on-the-fly, saving **~50% disk space**.

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

- **Auto Snapshots** — every file save captured
- **Branch Tracking** — history by Git branch
- **Semantic Deltas** — understands code structure
- **Instant Restore** — millisecond recovery
- **Full-Text Search** — search across all history
- **Deduplication** — 10-100x smaller than full copies
- **Symbol History** — track function/class evolution
- **IDE Integration** — open versions in your editor

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

MIT — See [LICENSE](LICENSE)
