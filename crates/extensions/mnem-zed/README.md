# Mnemosyne Zed Extension

This is the official Zed extension for **Mnemosyne**, providing intelligent local file history and semantic code navigation directly within Zed IDE.

## Features

- **Integrated Language Server**: Automatically manages and connects to `mnem-lsp`.
- **Seamless Multi-Language Support**: Adds historical context to Rust, Python, TypeScript, Go, and more without manual configuration.
- **Historical Hover**: Instantly see how many versions of a function or class exist in your local history.
- **Historical Goto Definition**: Navigate back in time to previous versions of any symbol.

## Prerequisites

1.  **Mnemosyne Daemon**: You must have `mnemd` installed and running.
    ```bash
    mnem start
    ```
2.  **LSP Binary**: Currently, this extension requires the `mnem-lsp` binary to be available in your system `PATH`.
    ```bash
    cargo install --path crates/mnem-lsp
    ```

## Installation (Local Development)

To use this extension in Zed while it is in development:

1.  Open Zed.
2.  Open the **Extensions** view (`Cmd+Shift+X` on macOS).
3.  Click **Install Dev Extension** at the top.
4.  Select the `crates/mnem-zed-extension` directory from this repository.
5.  Zed will compile the extension to WebAssembly and activate it.

## Configuration

The extension works out of the box. However, you can prioritize it or disable it for specific languages in your Zed `settings.json`:

```json
{
  "languages": {
    "Rust": {
      "language_servers": ["rust-analyzer", "mnem-lsp"]
    }
  }
}
```

## How it Works

The extension is written in Rust and compiled to **WebAssembly (Wasm)**. It uses the `zed_extension_api` to:
1.  Register `mnem-lsp` as a valid language server for multiple languages.
2.  Locate the `mnem-lsp` binary on your system.
3.  Orchestrate the communication between Zed and the Mnemosyne background daemon.

## Contributing

If you want to improve the extension:
- **Precise Extraction**: Help us move from word-based symbol detection to full Tree-sitter parsing within the Wasm environment.
- **Automatic Downloads**: Implement the logic to download pre-compiled `mnem-lsp` binaries for the user's platform automatically.

## License

Licensed under the MIT License.