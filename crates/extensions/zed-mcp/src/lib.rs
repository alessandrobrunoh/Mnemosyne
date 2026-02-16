use zed_extension_api::{self as zed, ContextServerId, Project};

struct MnemosyneMcpExtension;

impl zed::Extension for MnemosyneMcpExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> zed::Result<zed::Command> {
        // Try to find the mnem-mcp binary in multiple locations
        let binary_name = if cfg!(windows) {
            "mnem-mcp.exe"
        } else {
            "mnem-mcp"
        };

        // Locations to search (in order of preference)
        let search_paths = vec![
            // 1. Current directory
            std::path::PathBuf::from("."),
            // 2. Current working directory with binary name
            std::env::current_dir().unwrap_or_default(),
            // 3. Common development paths on Windows
            #[cfg(windows)]
            std::path::PathBuf::from(
                r"C:\Users\alexb\CodeProjects\Rust\Mnemosyne\mnemosyne\target\release",
            ),
            #[cfg(windows)]
            std::path::PathBuf::from(
                r"C:\Users\alexb\CodeProjects\Rust\Mnemosyne\mnemosyne\target\debug",
            ),
            // 4. PATH environment variable
        ];

        // First, try PATH
        if let Ok(path_env) = std::env::var("PATH") {
            let separator = if cfg!(windows) { ';' } else { ':' };
            for dir in path_env.split(separator) {
                let binary_path = std::path::PathBuf::from(dir).join(binary_name);
                if binary_path.exists() {
                    return Ok(zed::Command {
                        command: binary_path.to_string_lossy().into_owned(),
                        args: vec![],
                        env: vec![],
                    });
                }
            }
        }

        // Then try explicit paths
        for base_path in &search_paths {
            let binary_path = base_path.join(binary_name);
            if binary_path.exists() {
                return Ok(zed::Command {
                    command: binary_path.to_string_lossy().into_owned(),
                    args: vec![],
                    env: vec![],
                });
            }
        }

        // Try without .exe extension on Windows
        #[cfg(windows)]
        {
            let binary_path = std::path::PathBuf::from("mnem-mcp");
            if binary_path.exists() {
                return Ok(zed::Command {
                    command: binary_path.to_string_lossy().into_owned(),
                    args: vec![],
                    env: vec![],
                });
            }
        }

        Err(format!(
            "Mnemosyne MCP server ('{}') not found.\n\nPlease either:\n1. Add the binary to your PATH, or\n2. Build it with: cargo build --release -p mnem-mcp\n3. The binary should be at: target/release/mnem-mcp (or mnem-mcp.exe on Windows)",
            binary_name
        )
        .into())
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> zed::Result<Option<zed::ContextServerConfiguration>> {
        Ok(Some(zed::ContextServerConfiguration {
            installation_instructions: r#"# Mnemosyne MCP Server Installation

The Mnemosyne MCP server must be built and available.

## Quick Start

Run this command in the Mnemosyne project root:
```
cargo build --release -p mnem-mcp
```

This will create the binary at: `target/release/mnem-mcp` (or `mnem-mcp.exe` on Windows)

## Adding to PATH (Optional)

For permanent installation, add the binary to your system PATH.

## Usage

Once the binary is available, the MCP server will provide:
- Local file history tracking
- Semantic code navigation
- File change history
"#
            .to_string(),
            settings_schema: r#"{
  "type": "object",
  "properties": {}
}"#
            .to_string(),
            default_settings: r#"{}"#.to_string(),
        }))
    }
}

zed::register_extension!(MnemosyneMcpExtension);
