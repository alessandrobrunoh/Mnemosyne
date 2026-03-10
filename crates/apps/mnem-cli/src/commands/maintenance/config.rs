use anyhow::Result;
use clap::Args;
use serde::Serialize;
use serde_json::Value;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Presentable};

#[derive(Serialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ConfigData>,
}

#[derive(Serialize)]
pub struct ConfigData {
    pub storage: StorageConfig,
    pub ui: UiConfig,
    pub editor: EditorConfig,
}

#[derive(Serialize)]
pub struct StorageConfig {
    pub retention_days: u64,
    pub compression_enabled: bool,
    pub use_mnemosyneignore: bool,
    pub max_file_size_mb: u64,
}

#[derive(Serialize)]
pub struct UiConfig {
    pub theme_index: usize,
}

#[derive(Serialize)]
pub struct EditorConfig {
    pub ide: String,
}

impl Presentable for ConfigResponse {
    fn render_tui(&self) -> Result<()> {
        let layout = Layout::new();
        let theme = layout.theme();

        if let Some(config) = &self.config {
            layout.graph_branch_start("config: Mnemosyne");

            // Storage section
            layout.graph_connector();
            layout.graph_block_header("💾", "storage", theme.timeline_cyan);
            layout.graph_node(
                &config.storage.retention_days.to_string(),
                "RETENTION DAYS",
                false,
                "days",
                None,
                theme.text_dim,
            );
            layout.graph_node(
                &format!("{} MB", config.storage.max_file_size_mb),
                "MAX FILE SIZE",
                false,
                "limit",
                None,
                theme.text_dim,
            );
            layout.graph_node(
                &format!("{}", config.storage.compression_enabled),
                "COMPRESSION",
                false,
                if config.storage.compression_enabled {
                    "enabled"
                } else {
                    "disabled"
                },
                None,
                theme.text_dim,
            );
            layout.graph_node(
                &format!("{}", config.storage.use_mnemosyneignore),
                "USE IGNORE",
                false,
                if config.storage.use_mnemosyneignore {
                    "active"
                } else {
                    "inactive"
                },
                None,
                theme.text_dim,
            );

            // UI section
            layout.graph_connector();
            layout.graph_block_header("🎨", "ui", theme.success_bright);
            layout.graph_node(
                &config.ui.theme_index.to_string(),
                "THEME INDEX",
                false,
                "active",
                None,
                theme.text_dim,
            );

            // Editor section
            layout.graph_connector();
            layout.graph_block_header("✏️", "editor", theme.timeline_purple);
            layout.graph_node(
                &config.editor.ide,
                "PRIMARY IDE",
                false,
                "active",
                None,
                theme.text_dim,
            );

            layout.graph_branch_end();

            // Tips
            layout.empty();
            layout.badge_info(
                "USAGE",
                "mnem config --get <key> | --set <key>=<value> | --reset",
            );
            layout.badge_info("EXAMPLES", "mnem config --get storage.retention_days");
            layout.badge_info("EXAMPLES", "mnem config --set storage.retention_days=60");
        } else if self.success {
            layout.graph_branch_start("config: Mnemosyne");
            layout.graph_node(
                "SUCCESS",
                "STATUS",
                true,
                "updated",
                None,
                theme.success_bright,
            );
            layout.graph_branch_end();
            layout.empty();
            layout.info(&self.message);
        } else {
            layout.graph_branch_start("config: Mnemosyne");
            layout.graph_node(
                "ERROR",
                "STATUS",
                true,
                "failed",
                None,
                crossterm::style::Color::Red,
            );
            layout.graph_branch_end();
            layout.empty();
            layout.error(&self.message);
        }

        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Manage Mnemosyne configuration
#[derive(Args, Clone, Debug)]
pub struct ConfigCommand {
    /// Get a specific configuration value
    #[arg(short, long)]
    pub get: Option<String>,

    /// Set a specific configuration value (format: key=value)
    #[arg(short, long)]
    pub set: Option<String>,

    /// Reset configuration to defaults
    #[arg(long)]
    pub reset: bool,
}

impl CommandStrategy for ConfigCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        use mnem_core::config::ConfigManager;
        use mnem_core::env::get_base_dir;

        let base_dir = get_base_dir()?;
        let mut config_manager = ConfigManager::new(&base_dir)?;

        if self.reset {
            // Reset by removing the config file and letting it recreate with defaults
            let config_path = base_dir.join("config.toml");
            if config_path.exists() {
                std::fs::remove_file(&config_path)?;
            }
            config_manager = ConfigManager::new(&base_dir)?;

            let response = ConfigResponse {
                success: true,
                message: "Config reset to defaults".to_string(),
                config: None,
            };

            if global_opts.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&response.render_json()?)?
                );
            } else {
                response.render_tui()?;
            }
            return Ok(());
        }

        if let Some(ref key) = self.get {
            // Support both new nested keys and legacy flat keys
            let normalized_key = normalize_key(key.as_str());
            let value = config_manager
                .get_value(&normalized_key)
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            if global_opts.json {
                println!(
                    "{}",
                    serde_json::json!({ "success": true, "key": key, "value": value })
                );
            } else {
                let layout = Layout::new();
                let theme = layout.theme();
                layout.graph_branch_start(&format!("config: {}", key));
                layout.graph_node(&value, "VALUE", true, "current", None, theme.success_bright);
                layout.graph_branch_end();
            }
            return Ok(());
        }

        if let Some(ref key_value) = self.set {
            let parts: Vec<&str> = key_value.splitn(2, '=').collect();
            if parts.len() != 2 {
                let err_msg = "Usage: mnem config --set key=value".to_string();
                if global_opts.json {
                    println!(
                        "{}",
                        serde_json::json!({ "success": false, "error": err_msg })
                    );
                } else {
                    let layout = Layout::new();
                    layout.graph_branch_start("config: Mnemosyne");
                    layout.graph_node(
                        "ERROR",
                        "STATUS",
                        true,
                        "invalid",
                        None,
                        crossterm::style::Color::Red,
                    );
                    layout.graph_branch_end();
                    layout.empty();
                    layout.error(&err_msg);
                }
                return Ok(());
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            // Normalize the key and set the value
            let normalized_key = normalize_key(key);

            match config_manager.set_value(&normalized_key, value) {
                Ok(_) => {
                    let response = ConfigResponse {
                        success: true,
                        message: format!("Set {} = {}", key, value),
                        config: None,
                    };

                    if global_opts.json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&response.render_json()?)?
                        );
                    } else {
                        response.render_tui()?;
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to set config: {}", e);
                    if global_opts.json {
                        println!(
                            "{}",
                            serde_json::json!({ "success": false, "error": err_msg })
                        );
                    } else {
                        let layout = Layout::new();
                        layout.graph_branch_start("config: Mnemosyne");
                        layout.graph_node(
                            "ERROR",
                            "STATUS",
                            true,
                            "failed",
                            None,
                            crossterm::style::Color::Red,
                        );
                        layout.graph_branch_end();
                        layout.empty();
                        layout.error(&err_msg);
                    }
                }
            }
            return Ok(());
        }

        // Show full config
        let response = ConfigResponse {
            success: true,
            message: "Current configuration".to_string(),
            config: Some(ConfigData {
                storage: StorageConfig {
                    retention_days: config_manager.config.storage.retention_days,
                    compression_enabled: config_manager.config.storage.compression_enabled,
                    use_mnemosyneignore: config_manager.config.storage.use_mnemosyneignore,
                    max_file_size_mb: config_manager.config.storage.max_file_size_mb,
                },
                ui: UiConfig {
                    theme_index: config_manager.config.ui.theme_index,
                },
                editor: EditorConfig {
                    ide: config_manager.config.editor.ide.as_str().to_string(),
                },
            }),
        };

        if global_opts.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.render_json()?)?
            );
        } else {
            response.render_tui()?;
        }

        Ok(())
    }
}

/// Normalize legacy flat keys to new nested keys for backwards compatibility
fn normalize_key(key: &str) -> String {
    match key {
        // Legacy flat keys -> new nested keys
        "retention-days" | "retention_days" => "storage.retention_days".to_string(),
        "compression" | "compression_enabled" => "storage.compression_enabled".to_string(),
        "use-ignore" | "use_mnemosyneignore" => "storage.use_mnemosyneignore".to_string(),
        "max-file-size" | "max_file_size_mb" => "storage.max_file_size_mb".to_string(),
        "theme" | "theme_index" => "ui.theme_index".to_string(),
        "ide" => "editor.ide".to_string(),
        // Already nested keys - return as is
        _ => key.to_string(),
    }
}
