use anyhow::Result;
use clap::Args;
use serde::Serialize;
use serde_json::Value;
use std::path::Component;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Renderable};
use mnem_core::client::DaemonClient;
use mnem_core::protocol::SnapshotInfo;
use mnem_core::protocol::methods;
use mnem_core::storage::Repository;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Serialize)]
pub struct RestoreResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<SnapshotInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub limit: usize,
    pub page: usize,
}

impl Renderable for RestoreResponse {
    fn text(&self) -> Result<()> {
        use crate::ui::components::activity_graph::ActivityGraph;

        if let Some(history) = &self.history {
            if let Some(f) = &self.file {
                let cwd = std::env::current_dir()?;
                let mut graph = ActivityGraph::new(
                    &format!("RESTORE VERSIONS: {}", f),
                    history.clone(),
                    cwd,
                    Some(f.clone()),
                );
                graph.limit = self.limit;
                graph.page = self.page;
                graph.text()?;
                println!();
                Layout::new()
                    .info("Use 'mnem r <file> [version_number]' to restore a specific version.");
                return Ok(());
            }
        }

        if self.success {
            let layout = Layout::new();
            layout.header_dashboard("RESTORE");
            layout.success_bright(&self.message);
            layout.empty();
            if let Some(f) = &self.file {
                layout.info(&format!("File '{}' has been restored.", f));
            }
        } else {
            let layout = Layout::new();
            layout.header_dashboard("RESTORE FAILED");
            layout.error(&self.message);
        }

        Ok(())
    }
}

/// Restore files to previous versions
#[derive(Args, Clone, Debug)]
pub struct RestoreCommand {
    /// File to restore
    file: Option<String>,

    /// Version number to restore
    version: Option<usize>,

    /// List available versions
    #[arg(short, long)]
    list: bool,

    /// Undo last change
    #[arg(short, long)]
    undo: bool,

    /// Restore to specific hash
    #[arg(short, long)]
    to: Option<String>,

    /// Symbol to restore
    #[arg(short, long)]
    symbol: Option<String>,

    /// Restore to checkpoint
    #[arg(short, long)]
    checkpoint: Option<String>,

    /// Filter by git branch
    #[arg(short, long)]
    branch: Option<String>,

    /// Maximum number of results
    #[arg(short, long, default_value = "20")]
    limit: usize,

    /// Page number
    #[arg(short = 'P', long, default_value = "1")]
    page: usize,
}

impl CommandStrategy for RestoreCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        use mnem_core::config::ConfigManager;
        use mnem_core::env::get_base_dir;

        let base_dir = get_base_dir()?;
        let config = ConfigManager::new(&base_dir)?;
        let _ide = config.config.editor.ide;

        cleanup_old_temp_files();

        // Resolve project path
        let project_path = match get_project_from_file(&self.file) {
            Ok(p) => p,
            Err(_) => {
                if global_opts.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": "Project not tracked",
                            "code": "PROJECT_NOT_TRACKED"
                        })
                    );
                } else {
                    let layout = Layout::new();
                    let cwd = std::env::current_dir()?;
                    layout.header_dashboard("PROJECT NOT TRACKED");
                    layout.section_branch("pr", "Project Folder");
                    layout.row_labeled("◫", "Current Dir", &cwd.to_string_lossy());
                    layout.section_end();
                    layout.empty();
                    layout.badge_error("ERROR", "This project is not tracked");
                    layout.info_bright("Run 'mnem track' to start tracking this project.");
                }
                return Ok(());
            }
        };

        let daemon = DaemonClient::connect().ok();
        let repo_opt: Option<Repository> = if daemon.is_none() {
            match Repository::open(base_dir.clone(), project_path.clone()) {
                Ok(r) => Some(r),
                Err(_) => None,
            }
        } else {
            None
        };

        // --checkpoint
        if let Some(ref cp) = self.checkpoint {
            let message = if let Some(mut client) = daemon {
                let _ = client.call(
                    methods::PROJECT_REVERT_V1,
                    serde_json::json!({ "timestamp": cp }),
                )?;
                format!("Restored project from checkpoint {}", cp)
            } else if let Some(repo) = repo_opt.as_ref() {
                let count = repo.revert_to_checkpoint(cp)?;
                format!("Restored {} files from checkpoint {}", count, cp)
            } else {
                anyhow::bail!("Neither daemon nor local DB is available");
            };

            let response = RestoreResponse {
                success: true,
                message,
                history: None,
                file: None,
                limit: self.limit,
                page: self.page,
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response.json()?)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        let f = match self.file.as_ref() {
            Some(f) => f,
            None => {
                if global_opts.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": "Missing file path",
                            "code": "MISSING_FILE"
                        })
                    );
                } else {
                    Layout::new().error("Specify a file: mnem r <file> [version]");
                }
                return Ok(());
            }
        };

        let clean_path = f.trim_start_matches(".\\").trim_start_matches("./");

        // --list
        if self.list {
            let mut history = get_history_for_restore(
                daemon.as_ref().map(|_| ()),
                repo_opt.as_ref(),
                &project_path,
                clean_path,
                &mut DaemonClient::connect().ok(),
            )?;

            if let Some(ref br) = self.branch {
                history.retain(|s| s.git_branch.as_deref().unwrap_or("main") == br);
            }

            let response = RestoreResponse {
                success: true,
                message: "Success".to_string(),
                history: Some(history),
                file: Some(f.clone()),
                limit: self.limit,
                page: self.page,
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response.json()?)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        let do_restore = |daemon_opt: Option<DaemonClient>,
                          repo_ref: Option<&Repository>,
                          hash: &str,
                          sym: Option<&String>|
         -> Result<()> {
            let target = project_path.join(clean_path).to_string_lossy().to_string();
            if let Some(mut c) = daemon_opt {
                if let Some(s) = sym {
                    let _ = c.call(
                        methods::SNAPSHOT_RESTORE_SYMBOL_V1,
                        serde_json::json!({ "content_hash": hash, "target_path": target, "symbol_name": s }),
                    )?;
                } else {
                    let _ = c.call(
                        methods::SNAPSHOT_RESTORE_V1,
                        serde_json::json!({ "content_hash": hash, "target_path": target }),
                    )?;
                }
            } else if let Some(repo) = repo_ref {
                if let Some(s) = sym {
                    repo.restore_symbol(clean_path, hash, s)?;
                } else {
                    repo.restore_file(hash, clean_path)?;
                }
            } else {
                anyhow::bail!("Neither daemon nor local DB is available");
            }
            Ok(())
        };

        let mut message = String::new();

        // --undo
        if self.undo {
            let history = get_history_for_restore(
                daemon.as_ref().map(|_| ()),
                repo_opt.as_ref(),
                &project_path,
                clean_path,
                &mut DaemonClient::connect().ok(),
            )?;
            if history.len() < 2 {
                anyhow::bail!("No previous version to restore");
            }
            let prev_hash = history[1].content_hash.clone();
            let prev_ts = history[1].timestamp.clone();
            do_restore(daemon, repo_opt.as_ref(), &prev_hash, None)?;
            message = format!("Restored {} to version from {}", clean_path, prev_ts);
        } else if let Some(ref hash) = self.to {
            do_restore(daemon, repo_opt.as_ref(), hash, self.symbol.as_ref())?;
            message = if let Some(ref sym) = self.symbol {
                format!(
                    "Restored symbol '{}' in {} to {}",
                    sym,
                    clean_path,
                    &hash[..8.min(hash.len())]
                )
            } else {
                format!("Restored {} to {}", clean_path, &hash[..8.min(hash.len())])
            };
        } else if let Some(v) = self.version {
            let history = get_history_for_restore(
                daemon.as_ref().map(|_| ()),
                repo_opt.as_ref(),
                &project_path,
                clean_path,
                &mut DaemonClient::connect().ok(),
            )?;
            if v == 0 || v > history.len() {
                anyhow::bail!("Invalid version number. Use --list to see available versions.");
            }
            let target_hash = history[v - 1].content_hash.clone();
            do_restore(
                daemon,
                repo_opt.as_ref(),
                &target_hash,
                self.symbol.as_ref(),
            )?;
            message = if let Some(ref sym) = self.symbol {
                format!(
                    "Restored symbol '{}' in {} to version {}",
                    sym, clean_path, v
                )
            } else {
                format!("Restored {} to version {}", clean_path, v)
            };
        } else {
            // No action specified
            if !global_opts.json {
                let layout = Layout::new();
                layout.usage("r", "<file> [version]");
                layout.info("Examples:");
                layout.item_simple("mnem restore main.rs --list");
                layout.item_simple("mnem restore main.rs --list --branch test");
                layout.item_simple("mnem restore main.rs --list --limit 10");
                layout.item_simple("mnem restore main.rs 3");
                layout.item_simple("mnem restore main.rs --undo");
                layout.item_simple("mnem restore main.rs --to <hash>");
            }
            return Ok(());
        }

        let response = RestoreResponse {
            success: true,
            message,
            history: None,
            file: Some(f.clone()),
            limit: self.limit,
            page: self.page,
        };

        if global_opts.json {
            println!("{}", serde_json::to_string_pretty(&response.json()?)?);
        } else {
            response.text()?;
        }

        Ok(())
    }
}

fn cleanup_old_temp_files() {
    let cwd = std::env::current_dir().ok();
    if cwd.is_none() {
        return;
    }
    let cwd = cwd.unwrap();

    let entries = fs::read_dir(&cwd).ok();
    if entries.is_none() {
        return;
    }

    let now = SystemTime::now();
    let duration = Duration::from_secs(60 * 60); // 1 hour

    for entry in entries.unwrap() {
        let entry = entry.ok();
        if entry.is_none() {
            continue;
        }
        let entry = entry.unwrap();

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = path.file_name();
        if name.is_none() {
            continue;
        }
        let name = name.unwrap().to_string_lossy();

        // Look for .mnemosyne_restore_<timestamp>_<random> files
        if !name.starts_with(".mnemosyne_restore_") {
            continue;
        }

        let modified = fs::metadata(&path).ok().and_then(|m| m.modified().ok());
        if let Some(modified) = modified {
            if let Ok(age) = now.duration_since(modified) {
                if age > duration {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }
}

fn get_project_from_file(file: &Option<String>) -> Result<PathBuf> {
    if let Some(f) = file {
        if std::path::Path::new(f).is_absolute() {
            if let Some(parent) = std::path::Path::new(f).parent() {
                return Ok(parent.to_path_buf());
            }
        }
    }

    std::env::current_dir().map_err(|e| anyhow::anyhow!("Cannot get current directory: {}", e))
}

fn get_history_for_restore(
    _daemon: Option<()>,
    repo: Option<&Repository>,
    project_path: &PathBuf,
    file_path: &str,
    _client: &mut Option<DaemonClient>,
) -> Result<Vec<SnapshotInfo>> {
    let clean_path = file_path.trim_start_matches(".\\").trim_start_matches("./");
    let full_path = project_path.join(clean_path).to_string_lossy().to_string();

    if let Some(repo) = repo {
        let history = repo.get_history(&full_path)?;
        let infos = history
            .into_iter()
            .map(|s| mnem_core::protocol::SnapshotInfo {
                id: s.id,
                file_path: s.file_path,
                timestamp: s.timestamp,
                content_hash: s.content_hash,
                git_branch: s.git_branch,
                commit_hash: s.commit_hash,
                commit_message: s.commit_message,
                checkpoint_name: s.checkpoint_name,
            })
            .collect();
        return Ok(infos);
    }

    Ok(vec![])
}
