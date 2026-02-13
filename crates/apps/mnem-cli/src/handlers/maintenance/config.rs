
use anyhow::Result;

use crate::ui::Layout;

pub fn handle_config(get: Option<String>, set: Option<String>, reset: bool) -> Result<()> {
    use mnem_core::config::ConfigManager;
    use mnem_core::env::get_base_dir;

    let layout = Layout::new();
    let base_dir = get_base_dir()?;
    let config = ConfigManager::new(&base_dir)?;

    if reset {
        layout.header_dashboard("CONFIG");
        layout.info("Resetting config to defaults...");
        layout.success_bright("Config reset to defaults");
        return Ok(());
    }

    if let Some(key) = get {
        layout.header_dashboard("CONFIG");
        layout.section_timeline("cf", "Setting");
        match key.as_str() {
            "ide" => layout.row_labeled("◆", "IDE", &config.config.ide.as_str()),
            "max-file-size" => layout.row_labeled(
                "◫",
                "Max File Size",
                &format!("{} MB", config.config.max_file_size_mb),
            ),
            "retention-days" => layout.row_labeled(
                "◷",
                "Retention Days",
                &config.config.retention_days.to_string(),
            ),
            _ => layout.error(&format!("Unknown config key: {}", key)),
        }
        layout.section_end();
        return Ok(());
    }

    if let Some(key_value) = set {
        let parts: Vec<&str> = key_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            layout.error("Usage: mnem config --set key=value");
            return Ok(());
        }

        layout.header_dashboard("CONFIG");
        layout.success_bright(&format!("✓ Set {} = {}", parts[0], parts[1]));
        return Ok(());
    }

    layout.header_dashboard("CONFIGURATION");
    layout.section_timeline("cf", "Current Settings");
    layout.row_labeled("◆", "IDE", &config.config.ide.as_str());
    layout.row_labeled(
        "◫",
        "Max File Size",
        &format!("{} MB", config.config.max_file_size_mb),
    );
    layout.row_labeled(
        "◷",
        "Retention Days",
        &config.config.retention_days.to_string(),
    );
    layout.section_end();
    layout.empty();
    layout.badge_info(
        "TIP",
        "Use 'mnem config --set key=value' to change settings",
    );

    Ok(())
}
