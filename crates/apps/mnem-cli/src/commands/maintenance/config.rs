
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

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
    pub ide: String,
    pub max_file_size_mb: usize,
    pub retention_days: usize,
}

impl Presentable for ConfigResponse {
    fn render_tui(&self) -> Result<()> {
        let layout = Layout::new();
        if let Some(config) = &self.config {
            layout.header_dashboard("CONFIGURATION");
            layout.section_timeline("cf", "Current Settings");
            layout.row_labeled("◆", "IDE", &config.ide);
            layout.row_labeled(
                "◫",
                "Max File Size",
                &format!("{} MB", config.max_file_size_mb),
            );
            layout.row_labeled(
                "◷",
                "Retention Days",
                &config.retention_days.to_string(),
            );
            layout.section_end();
            layout.empty();
            layout.badge_info(
                "TIP",
                "Use 'mnem config --set key=value' to change settings",
            );
        } else if self.success {
            layout.success_bright(&self.message);
        } else {
            layout.error(&self.message);
        }
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_config(get: Option<String>, set: Option<String>, reset: bool, json: bool) -> Result<()> {
    use mnem_core::config::ConfigManager;
    use mnem_core::env::get_base_dir;

    let base_dir = get_base_dir()?;
    let config_manager = ConfigManager::new(&base_dir)?;

    if reset {
        let response = ConfigResponse {
            success: true,
            message: "Config reset to defaults".to_string(),
            config: None,
        };
        if json {
            println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
        } else {
            Layout::new().header_dashboard("CONFIG");
            Layout::new().info("Resetting config to defaults...");
            response.render_tui()?;
        }
        return Ok(());
    }

    if let Some(key) = get {
        let value = match key.as_str() {
            "ide" => config_manager.config.ide.as_str().to_string(),
            "max-file-size" => config_manager.config.max_file_size_mb.to_string(),
            "retention-days" => config_manager.config.retention_days.to_string(),
            _ => {
                let err_msg = format!("Unknown config key: {}", key);
                if json {
                    println!("{}", serde_json::json!({ "success": false, "error": err_msg }));
                } else {
                    Layout::new().error(&err_msg);
                }
                return Ok(());
            }
        };

        if json {
            println!("{}", serde_json::json!({ "success": true, "key": key, "value": value }));
        } else {
            let layout = Layout::new();
            layout.header_dashboard("CONFIG");
            layout.section_timeline("cf", "Setting");
            layout.row_labeled("◆", &key, &value);
            layout.section_end();
        }
        return Ok(());
    }

    if let Some(key_value) = set {
        let parts: Vec<&str> = key_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            if json {
                println!("{}", serde_json::json!({ "success": false, "error": "Usage: mnem config --set key=value" }));
            } else {
                Layout::new().error("Usage: mnem config --set key=value");
            }
            return Ok(());
        }

        let response = ConfigResponse {
            success: true,
            message: format!("Set {} = {}", parts[0], parts[1]),
            config: None,
        };

        if json {
            println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
        } else {
            Layout::new().header_dashboard("CONFIG");
            response.render_tui()?;
        }
        return Ok(());
    }

    let response = ConfigResponse {
        success: true,
        message: "Current configuration".to_string(),
        config: Some(ConfigData {
            ide: config_manager.config.ide.as_str().to_string(),
            max_file_size_mb: config_manager.config.max_file_size_mb as usize,
            retention_days: config_manager.config.retention_days as usize,
        }),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}
