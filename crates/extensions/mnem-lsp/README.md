# Mnemosyne LSP Server for Zed IDE

Language Server Protocol implementation for Mnemosyne integration with Zed IDE and other LSP-compatible editors.

## Features

- **Historical Hover**: Shows version count and last modification timestamp for symbols directly in the editor
- **Historical Goto Definition**: Navigate between historical versions of symbols across branches
- **Branch-Aware**: All operations respect the current Git branch context
- **Zero-Configuration**: Automatically detects and connects to the Mnemosyne daemon

## How It Works

The LSP server integrates with Mnemosyne's semantic analysis capabilities to provide historical context for your code:

1. When you hover over a symbol (function, class, variable), it queries Mnemosyne for that symbol's history
2. Displays the number of versions found in the current branch and the last modification timestamp
3. When using "Go to Definition", it provides a list of all historical versions of that symbol
4. Selecting a historical version opens that snapshot in the editor

## Prerequisites

- **Mnemosyne daemon** (`mnemd`) must be running and watching the current project
- **Tree-sitter grammars** must be available for your language (already included in Mnemosyne)
- **Project must be tracked** by Mnemosyne (use `mnem watch` in your project directory)

## Installation

### Build from Source

```bash
cd /path/to/mnemosyne
cargo build --release -p mnem-lsp
```

The binary will be available at `target/release/mnem-lsp`.

### System-Wide Installation

```bash
# Linux/macOS
sudo cp target/release/mnem-lsp /usr/local/bin/

# Windows
copy target\release\mnem-lsp.exe C:\Program Files\Mnemosyne\
```

## Zed IDE Configuration

### Step 1: Open Zed Settings

Press `Cmd+,` (macOS) or `Ctrl+,` (Linux/Windows) to open settings, or open `~/.config/zed/settings.json` manually.

### Step 2: Configure Language Server

Add the `mnem-lsp` configuration to your settings. The example below shows configuration for Rust, but you can adapt it for any supported language:

```json
{
  "languages": {
    "Rust": {
      "language_servers": ["rust-analyzer", "mnem-lsp"]
    },
    "Python": {
      "language_servers": ["pylsp", "mnem-lsp"]
    },
    "TypeScript": {
      "language_servers": ["typescript-language-server", "mnem-lsp"]
    }
  },
  "lsp": {
    "mnem-lsp": {
      "binary": {
        "path": "/usr/local/bin/mnem-lsp",
        "arguments": []
      }
    }
  }
}
```

### Step 3: Configure Multiple Languages

To enable Mnemosyne LSP for multiple languages simultaneously:

```json
{
  "languages": {
    "Rust": {
      "language_servers": ["rust-analyzer", "mnem-lsp"]
    },
    "Python": {
      "language_servers": ["pylsp", "mnem-lsp"]
    },
    "Go": {
      "language_servers": ["gopls", "mnem-lsp"]
    }
  },
  "lsp": {
    "mnem-lsp": {
      "binary": {
        "path": "/path/to/mnem-lsp",
        "arguments": []
      }
    }
  }
}
```

### Windows Configuration Example

```json
{
  "lsp": {
    "mnem-lsp": {
      "binary": {
        "path": "C:\\Program Files\\Mnemosyne\\mnem-lsp.exe",
        "arguments": []
      }
    }
  }
}
```

## Usage

### Enabling Mnemosyne for Your Project

Before using the LSP features, ensure your project is tracked by Mnemosyne:

```bash
# Navigate to your project
cd /path/to/your/project

# Start the Mnemosyne daemon (if not already running)
mnem start

# Watch the current project
mnem watch

# Verify the daemon is watching
mnem status
```

### Using Historical Hover

1. Open any tracked file in Zed
2. Hover over any symbol (function, class, variable)
3. The popup will show:
   - Number of versions found in the current branch
   - Last modification timestamp
   - Symbol type (function, class, etc.)

Example output:
```
Mnemosyne: `my_function` has 5 versions in branch `main`

Last modification: 2024-10-15T14:23:45Z
Type: function
```

### Using Historical Goto Definition

1. Place your cursor on any symbol
2. Press `F12` or `Cmd+Click` (macOS) / `Ctrl+Click` (Linux/Windows)
3. Zed will show a list of all historical versions
4. Select a version to open it in the editor

The list includes:
- Current version
- Previous versions with timestamps
- Original definition

## Troubleshooting

### Mnemosyne LSP Not Connecting

1. **Check daemon status**:
   ```bash
   mnem status
   ```
   
2. **Ensure project is watched**:
   ```bash
   mnem watch
   ```

3. **Check Zed logs**:
   - Open Command Palette: `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux/Windows)
   - Type: `zed: open log`
   - Look for errors related to `mnem-lsp`

### No Historical Information Available

- **Symbol not in database**: The symbol might be newly created and not yet indexed
- **Branch mismatch**: You might be looking at a different branch than the one indexed
- **Daemon not watching**: Ensure the project is being watched by Mnemosyne

### Performance Issues

If hover or goto definition is slow:

1. **Check daemon load**:
   ```bash
   mnem activity
   ```

2. **Reduce indexing scope**:
   - Use `.mnemosyneignore` to exclude unnecessary files
   - Reduce `max_file_size_mb` in configuration

3. **Adjust polling interval**:
   ```toml
   # In ~/.config/mnemosyne/config.toml
   monitor_interval_ms = 2000  # Increase from default 1000
   ```

### LSP Server Crashes

1. **Check logs**:
   ```bash
   # Run LSP server manually to see errors
   RUST_LOG=debug mnem-lsp
   ```

2. **Verify dependencies**:
   ```bash
   # Ensure all dependencies are installed
   cargo build --release -p mnem-lsp
   ```

## Architecture

The LSP server consists of:

- **Backend**: Main LSP handler implementing `LanguageServer` trait
- **Document Cache**: In-memory storage of opened documents for symbol extraction
- **Mnemosyne Client**: Communication layer with the `mnemd` daemon
- **Symbol Extraction**: Word-based extraction (can be enhanced with Tree-sitter)

### Communication Flow

```
Zed Editor
    |
    | LSP Protocol (stdio)
    v
mnem-lsp (LSP Server)
    |
    | JSON-RPC over Unix Socket
    v
mnemd (Mnemosyne Daemon)
    |
    | SQLite Queries
    v
Mnemosyne Database (~/.mnemosyne/projects/<id>.sqlite)
```

## Development

### Running in Debug Mode

```bash
cd crates/mnem-lsp
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run all tests
cargo test -p mnem-lsp

# Run with output
cargo test -p mnem-lsp -- --nocapture
```

### Building Documentation

```bash
cargo doc --open -p mnem-lsp
```

## Future Enhancements

- [ ] Tree-sitter integration for precise symbol extraction
- [ ] Semantic diff visualization in hover
- [ ] Custom command palette actions (restore, compare)
- [ ] Branch filtering in goto definition
- [ ] Symbol references across history
- [ ] Configuration via Zed settings (branch selection, history depth)

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) file for guidelines.

## License

Licensed under the MIT License. See [LICENSE](../../LICENSE) for details.