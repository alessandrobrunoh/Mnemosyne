use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{client::DaemonClient, protocol::methods, Repository};
use std::path::PathBuf;

fn resolve_path(arg: &str) -> Result<String> {
    let mut path = PathBuf::from(arg);
    if path.is_relative() {
        path = std::env::current_dir()?.join(path);
    }
    Ok(path.to_string_lossy().to_string())
}

#[derive(Debug)]
pub struct RestoreCommand;

impl Command for RestoreCommand {
    fn name(&self) -> &str {
        "restore"
    }

    fn usage(&self) -> &str {
        "<file_path> <content_hash> [--symbol <name>] or mnem restore --checkpoint <hash>"
    }

    fn description(&self) -> &str {
        "Restore a file or a symbol to a specific snapshot"
    }

    fn group(&self) -> &str {
        "Files"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        // Check for --checkpoint flag
        if let Some(pos) = args.iter().position(|a| a == "--checkpoint" || a == "-c") {
            if let Some(hash) = args.get(pos + 1) {
                let repo = Repository::init()?;
                let count = repo.revert_to_checkpoint(hash)?;

                layout.header("PROJECT RESTORED");
                layout.section_start("rs", "Massive Rollback");
                layout.item_simple(&format!(
                    "{} Reverted to checkpoint: {}",
                    "√".green(),
                    hash.clone().cyan().bold()
                ));
                layout.item_simple(&format!(
                    "{} Restored {} files.",
                    "→".cyan(),
                    count.to_string().white().bold()
                ));
                layout.section_end();
                return Ok(());
            }
        }

        if args.len() < 4 {
            layout.usage(self.name(), self.usage());
            return Ok(());
        }
        let file_path = resolve_path(&args[2])?;
        let hash = &args[3];

        // Check for --symbol flag
        let symbol_name = args
            .iter()
            .position(|a| a == "--symbol" || a == "-s")
            .and_then(|i| args.get(i + 1));

        let mut client = DaemonClient::connect()?;

        if let Some(symbol) = symbol_name {
            let _ = client.call(
                methods::SNAPSHOT_RESTORE_SYMBOL_V1,
                serde_json::json!({
                    "content_hash": hash,
                    "target_path": file_path,
                    "symbol_name": symbol,
                }),
            )?;

            layout.header("SURGICAL RESTORE");
            layout.section_start("rs", "Symbol Transplant");
            layout.item_simple(&format!(
                "{} Restored symbol '{}' from snapshot {}",
                "✓".green(),
                symbol.clone().bold().white(),
                hash[..8.min(hash.len())].cyan()
            ));
        } else {
            let _ = client.call(
                methods::SNAPSHOT_RESTORE_V1,
                serde_json::json!({
                    "content_hash": hash,
                    "target_path": file_path,
                }),
            )?;

            layout.header("RESTORE COMPLETED");
            layout.section_start("rs", "FileSystem Sync");
            layout.item_simple(&format!(
                "{} File restored: {} to {}",
                "✓".green(),
                hash.clone().bold().cyan(),
                file_path.white()
            ));
        }
        layout.section_end();
        Ok(())
    }
}
