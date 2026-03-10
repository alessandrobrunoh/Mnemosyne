use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::ui::{Layout, Presentable};
use mnem_core::client::DaemonClient;
use mnem_core::env::get_base_dir;
use mnem_core::protocol::SnapshotInfo;
use mnem_core::protocol::methods;
use mnem_core::storage::Repository;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

#[derive(Serialize)]
pub struct HistoryResponse {
    pub success: bool,
    pub project_name: String,
    pub history: Vec<SnapshotInfo>,
    pub file: Option<String>,
    pub limit: usize,
    pub page: usize,
}

impl Presentable for HistoryResponse {
    fn render_tui(&self) -> Result<()> {
        use crate::ui::components::activity_graph::ActivityGraph;
        let cwd = std::env::current_dir()?;
        
        let title = if let Some(ref f) = self.file {
            format!("FILE HISTORY: {}", f)
        } else {
            format!("PROJECT HISTORY: {}", self.project_name)
        };

        let mut graph = ActivityGraph::new(&title, self.history.clone(), cwd, self.file.clone());
        graph.limit = self.limit;
        graph.page = self.page;
        graph.render_tui()?;
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_h(
    file: Option<String>,
    limit: usize,
    page: usize,
    timeline: bool,
    _since: Option<String>,
    branch: Option<String>,
    clear: bool,
    json: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let project_path = cwd.clone();

    if clear {
        // Clear history functionality
        let mut success = false;
        let mut message = String::new();

        if mnem_core::client::daemon_running() {
            if let Ok(mut client) = DaemonClient::connect() {
                let params = mnem_core::protocol::ClearHistoryParams {
                    project_path: project_path.to_string_lossy().to_string(),
                };
                match client.call(methods::PROJECT_CLEAR_HISTORY, serde_json::to_value(params)?) {
                    Ok(res) => {
                        let cleared = res.get("cleared_snapshots").and_then(|v| v.as_u64()).unwrap_or(0);
                        success = true;
                        message = format!("Successfully cleared history ({} snapshots deleted)", cleared);
                    }
                    Err(e) => {
                        message = format!("Failed to clear history: {}", e);
                    }
                }
            }
        } else {
            // Daemon not running, clear manually if possible or report error
            message = "Daemon must be running to clear history".to_string();
        }

        if json {
            println!("{}", serde_json::json!({ "success": success, "message": message }));
        } else {
            let layout = Layout::new();
            if success {
                layout.empty();
                layout.badge_success("CLEARED", &message);
            } else {
                layout.empty();
                layout.badge_error("ERROR", &message);
            }
        }
        return Ok(());
    }

    let tracked_file = project_path.join(".mnemosyne").join("tracked");

    if !tracked_file.exists() {
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
            layout.header_dashboard("PROJECT NOT TRACKED");
            layout.section_branch("pr", "Project Path");
            layout.row_labeled("◫", "Path", &project_path.to_string_lossy());
            layout.section_end();
            layout.empty();
            layout.badge_error("ERROR", "This project is not tracked");
            layout.info_bright("Run 'mnem track' to start tracking this project.");
        }
        return Ok(());
    }

    if timeline {
        return handle_timeline_view(file, &Layout::new());
    }

    let project_name = if let Ok(content) = std::fs::read_to_string(&tracked_file) {
        content
            .lines()
            .find(|l| l.starts_with("project_name:"))
            .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        "Unknown".to_string()
    };

    let fetch_limit = 1000; // Get a large enough buffer for pagination

    let mut history = if let Some(ref f) = file {
        match try_daemon_file_history_data(f, fetch_limit, 0, &project_path) {
            Ok(h) => h,
            Err(_) => handle_file_history_direct_data(f, fetch_limit, 0, &project_path)?,
        }
    } else {
        match try_daemon_dashboard_view_data(fetch_limit, 0, &project_path) {
            Ok(h) => h,
            Err(_) => handle_dashboard_view_direct_data(fetch_limit, 0, &project_path)?,
        }
    };

    if let Some(ref br) = branch {
        history.retain(|s| s.git_branch.as_deref().unwrap_or("main") == br);
    }

    let response = HistoryResponse {
        success: true,
        project_name,
        history,
        file,
        limit,
        page,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}

fn try_daemon_file_history_data(
    f: &str,
    limit: usize,
    offset: usize,
    project_path: &std::path::Path,
) -> Result<Vec<SnapshotInfo>> {
    let mut client = DaemonClient::connect()?;
    let full_path = if std::path::Path::new(f).is_absolute() {
        f.to_string()
    } else {
        project_path.join(f).to_string_lossy().to_string()
    };

    let res = client.call(
        methods::SNAPSHOT_LIST,
        serde_json::json!({ 
            "file_path": full_path,
            "limit": limit,
            "offset": offset
        }),
    )?;

    Ok(serde_json::from_value(res)?)
}

fn handle_file_history_direct_data(
    f: &str,
    limit: usize,
    offset: usize,
    project_path: &std::path::Path,
) -> Result<Vec<SnapshotInfo>> {
    let base_dir = get_base_dir()?;
    let repo = Repository::open(base_dir, project_path.to_path_buf())?;

    let clean_path = if f.starts_with(".\\") {
        &f[2..]
    } else if f.starts_with("./") {
        &f[2..]
    } else {
        f
    };

    let absolute_path = if std::path::Path::new(clean_path).is_absolute() {
        clean_path.to_string()
    } else {
        project_path.join(clean_path).to_string_lossy().to_string()
    };

    // Note: Repository::get_history might need update to support limit/offset
    // For now we simulate it
    let history_db = repo.get_history(&absolute_path)?;
    
    Ok(history_db
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|sn| SnapshotInfo {
            id: sn.id,
            file_path: sn.file_path,
            timestamp: sn.timestamp,
            content_hash: sn.content_hash,
            git_branch: sn.git_branch,
            commit_hash: sn.commit_hash,
            commit_message: sn.commit_message,
            checkpoint_name: sn.checkpoint_name,
        })
        .collect())
}

fn try_daemon_dashboard_view_data(
    limit: usize,
    offset: usize,
    project_path: &std::path::Path,
) -> Result<Vec<SnapshotInfo>> {
    let mut client = DaemonClient::connect()?;

    let res = client.call(
        methods::PROJECT_GET_ACTIVITY,
        serde_json::json!({
            "limit": limit,
            "offset": offset,
            "project_path": project_path.to_string_lossy().to_string()
        }),
    )?;

    Ok(serde_json::from_value(res)?)
}

fn handle_dashboard_view_direct_data(
    limit: usize,
    offset: usize,
    project_path: &std::path::Path,
) -> Result<Vec<SnapshotInfo>> {
    let base_dir = get_base_dir()?;
    let repo = Repository::open(base_dir, project_path.to_path_buf())?;
    let history_db = repo.get_recent_activity(limit + offset)?;

    Ok(history_db
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|sn| SnapshotInfo {
            id: sn.id,
            file_path: sn.file_path,
            timestamp: sn.timestamp,
            content_hash: sn.content_hash,
            git_branch: sn.git_branch,
            commit_hash: sn.commit_hash,
            commit_message: sn.commit_message,
            checkpoint_name: sn.checkpoint_name,
        })
        .collect())
}

fn handle_timeline_view(file: Option<String>, layout: &Layout) -> Result<()> {
    if let Some(ref f) = file {
        layout.header_dashboard("TIMELINE");
        layout.section_branch("tl", f);
        layout.info("Timeline view coming soon");
        layout.section_end();
    } else {
        layout.error("Use --file to specify a file for timeline");
    }
    Ok(())
}

pub fn compute_diff_stats(
    repo: &Repository,
    current_hash: &str,
    prev_hash: Option<&str>,
) -> Option<(usize, usize)> {
    let current_content = repo.get_content(current_hash).ok()?;
    let prev_content = if let Some(p) = prev_hash {
        repo.get_content(p).ok().unwrap_or_default()
    } else {
        Vec::new()
    };

    let current_str = String::from_utf8_lossy(&current_content);
    let prev_str = String::from_utf8_lossy(&prev_content);

    let diff = TextDiff::from_lines(&prev_str, &current_str);

    let mut added = 0;
    let mut removed = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => added += 1,
            ChangeTag::Delete => removed += 1,
            _ => {}
        }
    }

    Some((added, removed))
}
