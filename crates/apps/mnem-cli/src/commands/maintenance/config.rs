use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{
    config::{ConfigManager, Ide},
    env::get_base_dir,
};

#[derive(Debug)]
pub struct ConfigCommand;

impl Command for ConfigCommand {
    fn name(&self) -> &str {
        "config"
    }

    fn usage(&self) -> &str {
        "set <key> <value>"
    }

    fn description(&self) -> &str {
        "Manage global Mnemosyne configuration"
    }

    fn group(&self) -> &str {
        "Maintenance"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        let base_dir = get_base_dir()?;
        let mut cm = ConfigManager::new(&base_dir)?;

        if args.len() == 2 {
            layout.section_start("cf", "Global Configuration");

            let settings = [
                ("Retention Days", cm.config.retention_days.to_string()),
                (
                    "Compression",
                    if cm.config.compression_enabled {
                        "enabled".green().to_string()
                    } else {
                        "disabled".red().to_string()
                    },
                ),
                (
                    "Max File Size",
                    format!("{} MB", cm.config.max_file_size_mb),
                ),
                (
                    "Use Gitignore",
                    if cm.config.use_gitignore {
                        "yes".green().to_string()
                    } else {
                        "no".red().to_string()
                    },
                ),
                (
                    "Use Mnemignore",
                    if cm.config.use_mnemosyneignore {
                        "yes".green().to_string()
                    } else {
                        "no".red().to_string()
                    },
                ),
                ("Theme Index", cm.config.theme_index.to_string()),
                (
                    "Primary IDE",
                    cm.config.ide.as_str().cyan().bold().to_string(),
                ),
            ];

            for (key, val) in settings {
                layout.row_property(key, &val);
            }
            layout.section_end();
            layout.footer("Use 'mnem config set <key> <val>' to update.");
        } else if args.len() >= 5 && args[2] == "set" {
            let key = &args[3];
            let val = &args[4];
            match key.as_str() {
                "retention_days" => cm.config.retention_days = val.parse()?,
                "compression" => cm.config.compression_enabled = val.parse()?,
                "max_file_size_mb" => cm.config.max_file_size_mb = val.parse()?,
                "use_gitignore" => cm.config.use_gitignore = val.parse()?,
                "use_mnemignore" => cm.config.use_mnemosyneignore = val.parse()?,
                "theme_index" => cm.config.theme_index = val.parse()?,
                "ide" => {
                    cm.config.ide = match val.to_lowercase().as_str() {
                        "zed" => Ide::Zed,
                        "zed-preview" | "zedpreview" => Ide::ZedPreview,
                        "vscode" | "code" => Ide::VsCode,
                        _ => {
                            eprintln!(
                                "{} Unknown IDE: {}. Use zed, zed-preview, or vscode.",
                                "✘".red(),
                                val
                            );
                            return Ok(());
                        }
                    };
                }
                _ => {
                    eprintln!("{} Unknown config key: {}", "✘".red(), key);
                    return Ok(());
                }
            }
            cm.save()?;
            println!("{} Config updated: {} = {}", "√".green(), key, val);
        } else {
            layout.usage(self.name(), self.usage());
        }
        Ok(())
    }
}
