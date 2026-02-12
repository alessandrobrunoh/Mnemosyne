use crate::ui::ButlerLayout;
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{
    client::DaemonClient,
    config::{ConfigManager, Ide},
    env::get_base_dir,
    protocol::methods,
    Repository,
};

pub fn config(args: &[String]) -> Result<()> {
    let base_dir = get_base_dir()?;
    let mut cm = ConfigManager::new(&base_dir)?;
    if args.len() == 2 {
        ButlerLayout::section_start("cf", "Global Configuration");

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
            let content = format!("{: <20} {}", key.white().dim(), val);
            ButlerLayout::row_list("•", &content);
        }
        ButlerLayout::section_end();
        ButlerLayout::footer("Use 'mnem config set <key> <val>' to update.");
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
        println!("Usage: mnem config set <key> <value>");
    }
    Ok(())
}

pub fn setup_protocol() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        ButlerLayout::section_start("os", "Protocol Registration");
        match crate::os::macos::install_url_handler() {
            Ok(_) => {
                ButlerLayout::item_simple(&format!(
                    "{} Protocol {} registered successfully!",
                    "√".green(),
                    "mnem://".bold().cyan()
                ));
                ButlerLayout::item_simple("Location: ~/Applications/MnemHandler.app");
                ButlerLayout::footer("You can now Shift+Click hashes in your terminal.");
            }
            Err(e) => {
                ButlerLayout::item_simple(&format!("{} Failed: {}", "✘".red(), e));
            }
        }
        ButlerLayout::section_end();
    }
    #[cfg(not(target_os = "macos"))]
    {
        println!("Protocol setup is currently only supported on macOS.");
    }
    Ok(())
}

pub fn gc() -> Result<()> {
    ButlerLayout::section_start("gc", "Garbage Collection");

    if let Ok(mut client) = DaemonClient::connect() {
        let res = client.call(methods::MAINTENANCE_GC, serde_json::json!({}))?;
        let pruned = res["pruned"].to_string();
        ButlerLayout::item_simple(&format!(
            "{} Successfully pruned {} orphan chunks.",
            "√".green(),
            pruned.bold()
        ));
    } else {
        let repo = Repository::init()?;
        let n = repo.run_gc()?;
        ButlerLayout::item_simple(&format!(
            "{} Local cleanup complete: {} chunks pruned.",
            "√".green(),
            n.to_string().bold()
        ));
    }
    ButlerLayout::section_end();
    Ok(())
}
