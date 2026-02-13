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

fn get_filename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

#[derive(Debug)]
pub struct InfoCommand;

impl Command for InfoCommand {
    fn name(&self) -> &str {
        "info"
    }

    fn usage(&self) -> &str {
        "<file_path>"
    }

    fn description(&self) -> &str {
        "Show detailed intelligence for a file"
    }

    fn group(&self) -> &str {
        "Files"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        if args.len() < 3 {
            layout.usage(self.name(), self.usage());
            return Ok(());
        }
        let file_path = resolve_path(&args[2])?;

        let mut client = DaemonClient::connect()?;
        let res = client.call(
            methods::FILE_GET_INFO,
            serde_json::json!({ "file_path": file_path }),
        )?;

        let info_val: serde_json::Value = serde_json::from_value(res)?;
        let filename = get_filename(&file_path);

        // Fetch checkpoint count for this file
        let checkpoint_count = if let Ok(repo) = Repository::init() {
            if let Ok(checkpoints) = repo.list_checkpoints() {
                checkpoints
                    .iter()
                    .filter(|(hash, _, _)| {
                        if let Ok(Some((_, file_states_json))) =
                            repo.db.get_checkpoint_by_hash(hash)
                        {
                            if let Ok(file_states) =
                                serde_json::from_str::<Vec<(String, String)>>(&file_states_json)
                            {
                                return file_states.iter().any(|(path, _)| *path == file_path);
                            }
                        }
                        false
                    })
                    .count()
            } else {
                0
            }
        } else {
            0
        };

        layout.header("FILE INTELLIGENCE");
        layout.section_start("st", &filename);

        let stats = [
            ("Path", info_val["path"].as_str().unwrap_or(&file_path)),
            ("Snapshots", &info_val["snapshot_count"].to_string()),
            ("Checkpoints", &checkpoint_count.to_string()),
            (
                "Size",
                &info_val["total_size_human"].as_str().unwrap_or("0 B"),
            ),
            ("Earliest", info_val["earliest"].as_str().unwrap_or("-")),
            ("Latest", info_val["latest"].as_str().unwrap_or("-")),
        ];

        for (idx, val) in stats {
            layout.row_property(idx, val);
        }
        layout.section_end();
        Ok(())
    }
}
