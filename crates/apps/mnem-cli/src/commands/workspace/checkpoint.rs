use anyhow::Result;
use clap::Args;
use serde::Serialize;
use std::collections::HashMap;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Renderable};
use mnem_core::client::DaemonClient;
use mnem_core::protocol::{self, methods};

#[derive(Args, Debug)]
#[command(alias = "cp")]
pub struct CheckpointCommand {
    /// Name of the checkpoint (e.g., 'v1-stable', 'pre-refactor')
    name: Option<String>,

    /// List all checkpoints
    #[arg(short, long)]
    list: bool,

    /// Remove the specified checkpoint
    #[arg(short, long)]
    remove: bool,

    /// Show checkpoint history (when it was created/updated)
    #[arg(long, alias = "hi")]
    history: bool,

    /// Show file manifest for the specified checkpoint
    #[arg(short, long)]
    show: bool,

    /// Update a specific file in the checkpoint to its current state
    #[arg(short, long)]
    update: Option<String>,

    /// Optional description for the checkpoint
    #[arg(short, long)]
    message: Option<String>,
}

#[derive(Serialize)]
pub struct CheckpointResponse {
    pub success: bool,
    pub message: String,
    pub checkpoints: Option<Vec<protocol::CheckpointInfo>>,
    pub manifest: Option<HashMap<String, String>>,
}

impl Renderable for CheckpointResponse {
    fn text(&self) -> Result<()> {
        let layout = Layout::new();

        if !self.success {
            layout.error(&self.message);
            return Ok(());
        }

        if let Some(ref cps) = self.checkpoints {
            layout.header_dashboard("PROJECT CHECKPOINTS");
            if cps.is_empty() {
                layout.info("No checkpoints found. Create one with 'mnem cp <name>'");
            } else {
                use crate::ui::Table;
                let mut table = Table::new(&["NAME", "BRANCH", "FILES", "CREATED", "DESCRIPTION"]);
                for cp in cps {
                    table.add_row(&[
                        &cp.name,
                        cp.git_branch.as_deref().unwrap_or("main"),
                        &cp.file_count.to_string(),
                        &cp.timestamp[..16].replace('T', " "),
                        cp.description.as_deref().unwrap_or("-"),
                    ]);
                }
                table.render();
            }
        } else if let Some(ref manifest) = self.manifest {
            layout.header_dashboard("CHECKPOINT MANIFEST");
            layout.info(&format!("Files in checkpoint: {}", manifest.len()));
            
            use crate::ui::components::elements::Hyperlink;
            use mnem_core::config::ConfigManager;
            use mnem_core::env::get_base_dir;

            let base_dir = get_base_dir()?;
            let config = ConfigManager::new(&base_dir)?.config;
            let ide = config.editor.ide;
            let temp_snapshots_dir = base_dir.join("snapshots");

            if !temp_snapshots_dir.exists() {
                let _ = std::fs::create_dir_all(&temp_snapshots_dir);
            }

            let mut client = DaemonClient::connect().ok();

            for (path, hash) in manifest {
                let file_name = std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file".to_string());

                let temp_file_name = format!("{}_{}", &hash[..8], file_name);
                let temp_path = temp_snapshots_dir.join(temp_file_name);

                let mut content_found = false;
                if temp_path.exists() {
                    content_found = true;
                } else if let Some(ref mut c) = client {
                    // Try to extract if missing
                    if let Ok(res) = c.call(
                        methods::SNAPSHOT_GET,
                        serde_json::json!({ "content_hash": hash }),
                    ) {
                        if let Some(s) = res["content"].as_str() {
                            if std::fs::write(&temp_path, s.as_bytes()).is_ok() {
                                content_found = true;
                            }
                        }
                    }
                }

                let link = if content_found {
                    Hyperlink::ide_link(&hash[..8], &temp_path.to_string_lossy(), &ide)
                } else {
                    hash[..8].to_string()
                };

                println!("  {}  {}", link, path);
            }
        } else {
            layout.success(&self.message);
        }

        Ok(())
    }
}

impl CommandStrategy for CheckpointCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        let mut client = DaemonClient::connect()?;

        // --list
        if self.list {
            let res = client.call(methods::PROJECT_LIST_CHECKPOINTS, serde_json::json!({}))?;
            let checkpoints: Vec<protocol::CheckpointInfo> = serde_json::from_value(res)?;

            let response = CheckpointResponse {
                success: true,
                message: "Success".to_string(),
                checkpoints: Some(checkpoints),
                manifest: None,
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        let name = match &self.name {
            Some(n) => n.clone(),
            None => {
                anyhow::bail!("Specify a checkpoint name (or use --list)");
            }
        };

        // --remove
        if self.remove {
            let _ = client.call(
                methods::PROJECT_REMOVE_CHECKPOINT,
                serde_json::json!({ "name": name }),
            )?;

            let response = CheckpointResponse {
                success: true,
                message: format!("Checkpoint '{}' removed", name),
                checkpoints: None,
                manifest: None,
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        // --show
        if self.show {
            let res = client.call(
                methods::PROJECT_GET_CHECKPOINT,
                serde_json::json!({ "name": name }),
            )?;
            let manifest_res: protocol::CheckpointManifestResponse = serde_json::from_value(res)?;

            let response = CheckpointResponse {
                success: true,
                message: "Success".to_string(),
                checkpoints: None,
                manifest: Some(manifest_res.file_states),
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        // --update <file>
        if let Some(ref file_path) = self.update {
            let _ = client.call(
                methods::PROJECT_UPDATE_CHECKPOINT_FILE,
                serde_json::json!({ "name": name, "file_path": file_path }),
            )?;

            let response = CheckpointResponse {
                success: true,
                message: format!("Updated file '{}' in checkpoint '{}'", file_path, name),
                checkpoints: None,
                manifest: None,
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        // Default: Create or Update
        let _ = client.call(
            methods::PROJECT_CREATE_CHECKPOINT,
            serde_json::json!({ "name": name, "description": self.message }),
        )?;

        let response = CheckpointResponse {
            success: true,
            message: format!("Checkpoint '{}' created/updated", name),
            checkpoints: None,
            manifest: None,
        };

        if global_opts.json {
            println!("{}", serde_json::to_string_pretty(&response)?);
        } else {
            response.text()?;
        }

        Ok(())
    }
}
