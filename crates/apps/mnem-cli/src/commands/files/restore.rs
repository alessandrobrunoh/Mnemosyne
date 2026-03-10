use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::path::Component;

use crate::ui::{Layout, Presentable};
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

impl Presentable for RestoreResponse {
    fn render_tui(&self) -> Result<()> {
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
                graph.render_tui()?;
                println!();
                Layout::new()
                    .info("Use 'mnem r <file> [version_number]' to restore a specific version.");
                return Ok(());
            }
        }

        if self.success {
            Layout::new().success_bright(&self.message);
        } else {
            Layout::new().error(&self.message);
        }
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// Resolve the project root from an optional file argument.
/// Looks for a `.mnemosyne/tracked` file walking upward from the file's parent dir.
fn get_project_from_file(file: &Option<String>) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let start = if let Some(f) = file {
        let p = std::path::Path::new(f);
        let resolved = if p.is_relative() {
            cwd.join(p)
        } else {
            p.to_path_buf()
        };
        resolved.parent().map(|p| p.to_path_buf()).unwrap_or(cwd)
    } else {
        cwd
    };

    let tracked_file = start.join(".mnemosyne").join("tracked");
    if !tracked_file.exists() {
        return Err(anyhow::anyhow!(
            "Project not tracked: {:?}\n\nRun 'mnem track' to start tracking this project.",
            start
        ));
    }
    Ok(start)
}

/// Normalize a path string by resolving `.` and `..` components without
/// touching the filesystem. This is needed on Windows where joining a
/// relative path like `.\file.txt` onto an absolute root produces a path
/// with a stray `CurDir` component (e.g. `C:\root\.\file.txt`) that
/// breaks both `Path::starts_with` comparisons and substring searches in
/// the database.
fn normalize_path(path: &str) -> String {
    let mut buf = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                buf.pop();
            }
            c => buf.push(c),
        }
    }
    buf.to_string_lossy().to_string()
}

fn cleanup_old_temp_files() {
    let temp_dir = std::env::temp_dir();
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(24 * 60 * 60))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        let name = entry.file_name();
                        if name.to_string_lossy().ends_with("_mnem.rs")
                            || name.to_string_lossy().ends_with("_mnem.tmp")
                        {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_r(
    file: Option<String>,
    version: Option<usize>,
    list: bool,
    undo: bool,
    to: Option<String>,
    symbol: Option<String>,
    checkpoint: Option<String>,
    branch: Option<String>,
    limit: usize,
    page: usize,
    json: bool,
) -> Result<()> {
    use mnem_core::config::ConfigManager;
    use mnem_core::env::get_base_dir;

    let base_dir = get_base_dir()?;
    let config = ConfigManager::new(&base_dir)?;
    let _ide = config.config.editor.ide;

    cleanup_old_temp_files();

    // Resolve project path
    let project_path = match get_project_from_file(&file) {
        Ok(p) => p,
        Err(_) => {
            if json {
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
    if let Some(ref cp) = checkpoint {
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
            limit,
            page,
        };

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.render_json()?)?
            );
        } else {
            response.render_tui()?;
        }
        return Ok(());
    }

    let f = match file.as_ref() {
        Some(f) => f,
        None => {
            if json {
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
    if list {
        let mut history = get_history_for_restore(
            daemon.as_ref().map(|_| ()),
            repo_opt.as_ref(),
            &project_path,
            clean_path,
            &mut DaemonClient::connect().ok(),
        )?;

        if let Some(ref br) = branch {
            history.retain(|s| s.git_branch.as_deref().unwrap_or("main") == br);
        }

        let response = RestoreResponse {
            success: true,
            message: "Success".to_string(),
            history: Some(history),
            file: Some(f.clone()),
            limit,
            page,
        };

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.render_json()?)?
            );
        } else {
            response.render_tui()?;
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
    if undo {
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
    } else if let Some(ref hash) = to {
        do_restore(daemon, repo_opt.as_ref(), hash, symbol.as_ref())?;
        message = if let Some(ref sym) = symbol {
            format!(
                "Restored symbol '{}' in {} to {}",
                sym,
                clean_path,
                &hash[..8.min(hash.len())]
            )
        } else {
            format!("Restored {} to {}", clean_path, &hash[..8.min(hash.len())])
        };
    } else if let Some(v) = version {
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
        do_restore(daemon, repo_opt.as_ref(), &target_hash, symbol.as_ref())?;
        message = if let Some(ref sym) = symbol {
            format!(
                "Restored symbol '{}' in {} to version {}",
                sym, clean_path, v
            )
        } else {
            format!("Restored {} to version {}", clean_path, v)
        };
    } else {
        // No action specified
        if !json {
            let layout = Layout::new();
            layout.usage("r", "<file> [version]");
            layout.info("Examples:");
            layout.item_simple("mnem r main.rs --list");
            layout.item_simple("mnem r main.rs --list --branch test");
            layout.item_simple("mnem r main.rs --list --limit 10");
            layout.item_simple("mnem r main.rs 3");
            layout.item_simple("mnem r main.rs --undo");
            layout.item_simple("mnem r main.rs --to <hash>");
        }
        return Ok(());
    }

    let response = RestoreResponse {
        success: true,
        message,
        history: None,
        file: Some(f.clone()),
        limit,
        page,
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&response.render_json()?)?
        );
    } else {
        response.render_tui()?;
    }

    Ok(())
}

/// Get snapshot history, preferring daemon then falling back to direct DB.
fn get_history_for_restore(
    daemon_present: Option<()>,
    repo_opt: Option<&Repository>,
    project_path: &PathBuf,
    clean_path: &str,
    client: &mut Option<DaemonClient>,
) -> Result<Vec<SnapshotInfo>> {
    if daemon_present.is_some() {
        if let Some(c) = client.as_mut() {
            let full_path = project_path.join(clean_path).to_string_lossy().to_string();
            let res = c.call(
                methods::SNAPSHOT_LIST,
                serde_json::json!({ "file_path": full_path }),
            )?;
            match serde_json::from_value::<Vec<SnapshotInfo>>(res.clone()) {
                Ok(history) => return Ok(history),
                Err(e) => {
                    eprintln!("Daemon parse error in get_history_for_restore: {}", e);
                    eprintln!("Raw response: {:?}", res);
                }
            }
        }
    }
    if let Some(repo) = repo_opt {
        let snaps = repo.get_history(clean_path)?;
        return Ok(snaps
            .into_iter()
            .map(|s| SnapshotInfo {
                id: s.id,
                file_path: s.file_path,
                timestamp: s.timestamp,
                content_hash: s.content_hash,
                git_branch: s.git_branch,
                commit_hash: s.commit_hash,
                commit_message: s.commit_message,
                checkpoint_name: s.checkpoint_name,
            })
            .collect());
    }
    anyhow::bail!("Neither daemon nor local DB is available to fetch history")
}
