use anyhow::Result;
use clap::Args;
use serde::Serialize;
use serde_json::Value;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Presentable};
use mnem_core::client::DaemonClient;
use mnem_core::env::get_base_dir;
use mnem_core::protocol::SnapshotInfo;
use mnem_core::protocol::methods;
use mnem_core::storage::Repository;
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

        let layout = Layout::new();
        layout.header_dashboard(&title);

        if self.history.is_empty() {
            layout.info("No history available for this file");
            return Ok(());
        }

        // Show activity graph
        let graph =
            ActivityGraph::new(&title, self.history.clone(), cwd.clone(), self.file.clone());
        let _ = graph.render_tui();

        layout.empty();

        // Group history by date
        let mut grouped: std::collections::HashMap<String, Vec<&SnapshotInfo>> =
            std::collections::HashMap::new();

        for snapshot in &self.history {
            let date = &snapshot.timestamp[..10]; // YYYY-MM-DD
            grouped.entry(date.to_string()).or_default().push(snapshot);
        }

        // Paginate
        let total_items = self.history.len();
        let total_pages = (total_items as f64 / self.limit as f64).ceil() as usize;
        let offset = (self.page.saturating_sub(1)) * self.limit;

        let mut sorted_dates: Vec<String> = grouped.keys().cloned().collect();
        sorted_dates.sort();
        sorted_dates.reverse();

        let mut count = 0;
        for date in sorted_dates {
            if count < offset {
                count += grouped[&date].len();
                continue;
            }

            if count >= offset + self.limit {
                break;
            }

            let snapshots = &grouped[&date];
            layout.section_start("hi", &format!("{} - {} snapshots", date, snapshots.len()));

            for snap in snapshots {
                let time = &snap.timestamp[11..19]; // HH:MM:SS
                let short_hash = &snap.content_hash[..8];

                let clickable_hash =
                    crate::ui::Hyperlink::action(short_hash, "open", &snap.content_hash);

                let meta = format!(
                    "{} | {}",
                    time,
                    snap.git_branch.as_deref().unwrap_or("main").to_string()
                );

                layout.row_snapshot(&clickable_hash, &meta);
            }

            layout.section_end();
            count += snapshots.len();
        }

        if total_pages > 1 {
            layout.empty();
            layout.info(&format!("Page {} of {}", self.page, total_pages));
        }

        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

/// View and manage file history
#[derive(Args, Clone, Debug)]
pub struct HistoryCommand {
    /// Specific file to view history for
    file: Option<String>,

    /// Maximum number of results
    #[arg(short, long, default_value = "20")]
    limit: usize,

    /// Page number
    #[arg(short = 'P', long, default_value = "1")]
    page: usize,

    /// Show timeline view
    #[arg(short, long)]
    timeline: bool,

    /// Since date
    #[arg(short, long)]
    _since: Option<String>,

    /// Filter by git branch
    #[arg(short, long)]
    branch: Option<String>,

    /// Clear all history for current project
    #[arg(short, long)]
    clear: bool,
}

impl CommandStrategy for HistoryCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let project_path = cwd.clone();

        if self.clear {
            return self.clear_history(&project_path, global_opts.json);
        }

        let tracked_file = project_path.join(".mnemosyne").join("tracked");

        if !tracked_file.exists() {
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

        if self.timeline {
            return handle_timeline_view(self.file.as_deref(), &Layout::new());
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

        let fetch_limit = 1000;

        let mut history = if let Some(ref f) = self.file {
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

        if let Some(ref br) = self.branch {
            history.retain(|s| s.git_branch.as_deref().unwrap_or("main") == br);
        }

        let response = HistoryResponse {
            success: true,
            project_name,
            history,
            file: self.file.clone(),
            limit: self.limit,
            page: self.page,
        };

        if global_opts.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.render_json()?)?
            );
        } else {
            response.render_tui()?;
        }

        Ok(())
    }
}

impl HistoryCommand {
    fn clear_history(&self, project_path: &PathBuf, json: bool) -> Result<()> {
        let mut success = false;
        let mut message = String::new();

        if mnem_core::client::daemon_running() {
            if let Ok(mut client) = DaemonClient::connect() {
                let params = mnem_core::protocol::ClearHistoryParams {
                    project_path: project_path.to_string_lossy().to_string(),
                };
                match client.call(
                    methods::PROJECT_CLEAR_HISTORY,
                    serde_json::to_value(params)?,
                ) {
                    Ok(res) => {
                        let cleared = res
                            .get("cleared_snapshots")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        success = true;
                        message = format!(
                            "Successfully cleared history ({} snapshots deleted)",
                            cleared
                        );
                    }
                    Err(e) => {
                        message = format!("Failed to clear history: {}", e);
                    }
                }
            }
        } else {
            message = "Daemon must be running to clear history".to_string();
        }

        if json {
            println!(
                "{}",
                serde_json::json!({ "success": success, "message": message })
            );
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

        Ok(())
    }
}

fn handle_timeline_view(file: Option<&str>, layout: &Layout) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let base_dir = get_base_dir()?;
    let project_path = cwd.clone();

    let repo = match Repository::open(base_dir, project_path) {
        Ok(r) => r,
        Err(_) => {
            layout.error("Cannot open repository");
            return Ok(());
        }
    };

    let history = repo.get_recent_activity(50)?;

    if history.is_empty() {
        layout.info("No recent activity");
        return Ok(());
    }

    layout.header_dashboard("TIMELINE VIEW");

    // Group by file
    let mut file_activity: std::collections::HashMap<String, Vec<&SnapshotInfo>> =
        std::collections::HashMap::new();

    let history_infos: Vec<SnapshotInfo> = history
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
        .collect();

    for snap in history_infos.iter() {
        file_activity
            .entry(snap.file_path.clone())
            .or_default()
            .push(snap);
    }

    let mut files: Vec<&String> = file_activity.keys().collect();
    files.sort();

    for file_path in files {
        let snapshots = &file_activity[file_path];
        let filename = std::path::Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.clone());

        layout.section_start("tl", &filename);

        for snap in snapshots.iter().take(5) {
            let time = &snap.timestamp[11..19];
            let short_hash = &snap.content_hash[..8];

            let clickable_hash =
                crate::ui::Hyperlink::action(short_hash, "open", &snap.content_hash);

            layout.row_snapshot(&clickable_hash, time);
        }

        if snapshots.len() > 5 {
            layout.info(&format!("+ {} more changes", snapshots.len() - 5));
        }

        layout.section_end();
    }

    Ok(())
}

fn try_daemon_file_history_data(
    f: &str,
    limit: usize,
    offset: usize,
    project_path: &PathBuf,
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
    project_path: &PathBuf,
) -> Result<Vec<SnapshotInfo>> {
    let base_dir = get_base_dir()?;
    let full_path = if std::path::Path::new(f).is_absolute() {
        f.to_string()
    } else {
        project_path.join(f).to_string_lossy().to_string()
    };

    let repo = Repository::open(base_dir, project_path.clone())?;

    let history = repo.get_history(&full_path)?;
    let history = history
        .into_iter()
        .skip(offset)
        .take(limit)
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
        .collect();

    Ok(history)
}

fn try_daemon_dashboard_view_data(
    limit: usize,
    offset: usize,
    project_path: &PathBuf,
) -> Result<Vec<SnapshotInfo>> {
    let mut client = DaemonClient::connect()?;

    let res = client.call(
        methods::PROJECT_GET_ACTIVITY,
        serde_json::json!({
            "project_path": project_path.to_string_lossy(),
            "limit": limit,
            "offset": offset
        }),
    )?;

    Ok(serde_json::from_value(res)?)
}

fn handle_dashboard_view_direct_data(
    limit: usize,
    offset: usize,
    project_path: &PathBuf,
) -> Result<Vec<SnapshotInfo>> {
    let base_dir = get_base_dir()?;
    let repo = Repository::open(base_dir, project_path.clone())?;
    let history = repo.get_recent_activity(limit + offset)?;
    let history = history
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|snap| SnapshotInfo {
            id: snap.id,
            file_path: snap.file_path,
            timestamp: snap.timestamp,
            content_hash: snap.content_hash,
            git_branch: snap.git_branch,
            commit_hash: snap.commit_hash,
            commit_message: snap.commit_message,
            checkpoint_name: snap.checkpoint_name,
        })
        .collect();
    Ok(history)
}
