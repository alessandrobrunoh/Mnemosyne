use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{client::DaemonClient, protocol::methods};
use std::path::PathBuf;

fn resolve_path(arg: &str) -> Result<String> {
    let mut path = PathBuf::from(arg);
    if path.is_relative() {
        path = std::env::current_dir()?.join(path);
    }
    Ok(path.to_string_lossy().to_string())
}

fn get_filename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

#[derive(Debug)]
pub struct DiffCommand;

impl Command for DiffCommand {
    fn name(&self) -> &str {
        "diff"
    }

    fn usage(&self) -> &str {
        "<file_path> <hash1> [hash2]"
    }

    fn description(&self) -> &str {
        "Show differences between file versions"
    }

    fn group(&self) -> &str {
        "Files"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        if args.len() < 4 {
            layout.usage(self.name(), self.usage());
            return Ok(());
        }
        let file_path = resolve_path(&args[2])?;

        let mut client = DaemonClient::connect()?;
        let res = client.call(
            methods::SNAPSHOT_LIST,
            serde_json::json!({ "file_path": file_path }),
        )?;

        let history: Vec<mnem_core::protocol::SnapshotInfo> = serde_json::from_value(res)?;
        if history.is_empty() {
            println!("No history found.");
            return Ok(());
        }

        let hash1 = &args[3];
        let (hash2, is_disk) = if args.len() > 4 {
            (args[4].clone(), false)
        } else {
            ("__DISK__".to_string(), true)
        };

        let res = client.call(
            methods::FILE_GET_DIFF,
            serde_json::json!({
                "file_path": file_path,
                "base_hash": Some(hash1),
                "target_hash": hash2,
            }),
        )?;

        let diff_res: mnem_core::protocol::FileDiffResponse = serde_json::from_value(res)?;
        let filename = get_filename(&file_path);

        layout.header("FILE COMPARISON");
        layout.section_start("di", &filename);
        let base_name = if is_disk {
            "Current Disk".yellow()
        } else {
            hash2[..8.min(hash2.len())].blue().bold()
        };
        layout.item_simple(&format!(
            "Comparing: {} -> {}",
            hash1[..8.min(hash1.len())].blue().bold(),
            base_name
        ));
        println!("┊");

        for line in diff_res.diff.lines() {
            let styled_line = if line.starts_with('+') {
                line.green()
            } else if line.starts_with('-') {
                line.red()
            } else if line.starts_with("@@") {
                line.cyan().dim()
            } else {
                line.stylize()
            };
            layout.item_simple(&styled_line.to_string());
        }
        layout.section_end();
        Ok(())
    }
}
