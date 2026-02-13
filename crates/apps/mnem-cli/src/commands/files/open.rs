use crate::commands::Command;
use crate::ui::{self, Layout};
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{client::DaemonClient, protocol::methods, Repository};

#[derive(Debug)]
pub struct OpenCommand;

impl Command for OpenCommand {
    fn name(&self) -> &str {
        "open"
    }

    fn usage(&self) -> &str {
        "<hash>"
    }

    fn description(&self) -> &str {
        "Open a snapshot in your configured IDE"
    }

    fn group(&self) -> &str {
        "Files"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["o"]
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        if args.len() < 3 {
            layout.usage(self.name(), self.usage());
            return Ok(());
        }
        let hash = &args[2];

        // Try to resolve the repository and original file path to get the correct extension
        let repo = Repository::find_by_hash(hash)?;
        let snapshot_info = repo
            .db
            .get_history_by_hash(hash)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Could not resolve snapshot info for hash {}", hash))?;

        let extension = std::path::Path::new(&snapshot_info.file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");

        // Ensure daemon is running so we can talk to it
        let _ = mnem_core::client::ensure_daemon();
        let mut client = DaemonClient::connect()?;
        let res = client.call(
            methods::SNAPSHOT_GET,
            serde_json::json!({ "content_hash": hash }),
        )?;
        let content_res: serde_json::Value = serde_json::from_value(res)?;
        let content = content_res["content"].as_str().unwrap_or("");

        let hash_prefix = if hash.len() > 8 { &hash[..8] } else { hash };
        // Use a more portable way to handle temp dir
        let tmp_path =
            std::env::temp_dir().join(format!("mnem_snap_{}.{}", hash_prefix, extension));
        std::fs::write(&tmp_path, content)?;

        let ide = {
            let config_manager = repo
                .config
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            config_manager.config.ide
        };

        layout.header("OPENING SNAPSHOT");
        layout.item_simple(&format!(
            "{} Target IDE: {}",
            "→".cyan(),
            ide.as_str().to_string().bold().white()
        ));
        layout.item_simple(&format!(
            "{} Hash: {}",
            "→".cyan(),
            hash.clone().with(ui::ACCENT).bold()
        ));

        #[cfg(target_os = "macos")]
        let mut cmd = std::process::Command::new("open");
        #[cfg(target_os = "windows")]
        let mut cmd = std::process::Command::new("explorer");
        #[cfg(target_os = "linux")]
        let mut cmd = std::process::Command::new("xdg-open");

        #[cfg(target_os = "macos")]
        match ide {
            mnem_core::config::Ide::Zed => {
                cmd.arg("-a").arg("Zed");
            }
            mnem_core::config::Ide::VsCode => {
                cmd.arg("-a").arg("Visual Studio Code");
            }
            mnem_core::config::Ide::ZedPreview => {
                cmd.arg("-a").arg("Zed Preview");
            }
        }

        cmd.arg(&tmp_path);

        if let Err(e) = cmd.spawn() {
            eprintln!("{} Failed to launch IDE: {}", "✘".red(), e);
        }

        Ok(())
    }
}
