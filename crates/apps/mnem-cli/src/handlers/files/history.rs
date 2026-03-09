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
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct HistoryResponse {
    pub success: bool,
    pub project_name: String,
    pub history: Vec<SnapshotInfo>,
    pub file: Option<String>,
    pub limit: usize,
}

impl Presentable for HistoryResponse {
    fn render_tui(&self) -> Result<()> {
        let layout = Layout::new();
        if let Some(ref f) = self.file {
            display_file_history(f, self.limit, &layout, self.history.clone())?;
        } else {
            display_dashboard_view(
                self.limit,
                &layout,
                &std::env::current_dir()?,
                self.project_name.clone(),
                self.history.clone(),
            )?;
        }
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

fn check_project_tracked(layout: &Layout) -> Result<(PathBuf, PathBuf)> {
    let base_dir = get_base_dir()?;
    let cwd = std::env::current_dir()?;
    let tracked_file = cwd.join(".mnemosyne").join("tracked");

    if !tracked_file.exists() {
        layout.header_dashboard("PROJECT NOT TRACKED");
        layout.section_branch("pr", "Current Folder");
        layout.row_labeled("◫", "Path", &cwd.to_string_lossy());
        layout.section_end();
        layout.empty();
        layout.badge_error("ERROR", "This project is not tracked");
        layout.info_bright("Run 'mnem track' to start tracking this project.");
        anyhow::bail!("Project not tracked");
    }

    Ok((base_dir, cwd))
}

pub fn handle_h(
    file: Option<String>,
    limit: Option<usize>,
    timeline: bool,
    since: Option<String>,
    branch: Option<String>,
    json: bool,
) -> Result<()> {
    let limit = limit.unwrap_or(20);
    let cwd = std::env::current_dir()?;
    let project_path = cwd.clone();
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

    let history = if let Some(ref f) = file {
        match try_daemon_file_history_data(f, limit, &project_path) {
            Ok(h) => h,
            Err(_) => handle_file_history_direct_data(f, limit, &project_path)?,
        }
    } else {
        match try_daemon_dashboard_view_data(limit, &project_path) {
            Ok(h) => h,
            Err(_) => handle_dashboard_view_direct_data(limit, &project_path)?,
        }
    };

    let response = HistoryResponse {
        success: true,
        project_name,
        history,
        file,
        limit,
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
    _limit: usize,
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
        serde_json::json!({ "file_path": full_path }),
    )?;

    Ok(serde_json::from_value(res)?)
}

fn handle_file_history_direct_data(
    f: &str,
    _limit: usize,
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

    let history_db = repo.get_history(&absolute_path)?;

    Ok(history_db
        .into_iter()
        .map(|sn| SnapshotInfo {
            id: sn.id,
            file_path: sn.file_path,
            timestamp: sn.timestamp,
            content_hash: sn.content_hash,
            git_branch: sn.git_branch,
            commit_hash: sn.commit_hash,
            commit_message: None,
        })
        .collect())
}

fn try_daemon_dashboard_view_data(
    limit: usize,
    project_path: &std::path::Path,
) -> Result<Vec<SnapshotInfo>> {
    let mut client = DaemonClient::connect()?;

    let res = client.call(
        methods::PROJECT_GET_ACTIVITY,
        serde_json::json!({
            "limit": limit,
            "project_path": project_path.to_string_lossy().to_string()
        }),
    )?;

    Ok(serde_json::from_value(res)?)
}

fn handle_dashboard_view_direct_data(
    limit: usize,
    project_path: &std::path::Path,
) -> Result<Vec<SnapshotInfo>> {
    let base_dir = get_base_dir()?;
    let repo = Repository::open(base_dir, project_path.to_path_buf())?;
    let history_db = repo.get_recent_activity(limit)?;

    Ok(history_db
        .into_iter()
        .map(|sn| SnapshotInfo {
            id: sn.id,
            file_path: sn.file_path,
            timestamp: sn.timestamp,
            content_hash: sn.content_hash,
            git_branch: sn.git_branch,
            commit_hash: sn.commit_hash,
            commit_message: None,
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

// Try to use daemon for file history
fn try_daemon_file_history(
    f: &str,
    limit: usize,
    layout: &Layout,
    project_path: &std::path::Path,
) -> Result<()> {
    let mut client = DaemonClient::connect()?;
    let full_path = if std::path::Path::new(f).is_absolute() {
        f.to_string()
    } else {
        project_path.join(f).to_string_lossy().to_string()
    };

    let res = client.call(
        methods::SNAPSHOT_LIST,
        serde_json::json!({ "file_path": full_path }),
    )?;

    let history: Vec<SnapshotInfo> = serde_json::from_value(res)?;
    display_file_history(f, limit, layout, history)
}

// Fallback: direct database access for file history
fn handle_file_history_direct(
    f: &str,
    limit: usize,
    layout: &Layout,
    project_path: &std::path::Path,
) -> Result<()> {
    let base_dir = get_base_dir()?;
    let repo = Repository::open(base_dir, project_path.to_path_buf())?;

    // Convert relative path to absolute by joining with project_path
    let clean_path = if f.starts_with(".\\") {
        &f[2..]
    } else if f.starts_with("./") {
        &f[2..]
    } else {
        f
    };

    // Use absolute path for database lookup (how files are stored internally)
    let absolute_path = if std::path::Path::new(clean_path).is_absolute() {
        clean_path.to_string()
    } else {
        project_path.join(clean_path).to_string_lossy().to_string()
    };

    let history_db = repo.get_history(&absolute_path)?;

    // Convert to SnapshotInfo format
    let history: Vec<SnapshotInfo> = history_db
        .into_iter()
        .map(|sn| SnapshotInfo {
            id: sn.id,
            file_path: sn.file_path,
            timestamp: sn.timestamp,
            content_hash: sn.content_hash,
            git_branch: sn.git_branch,
            commit_hash: sn.commit_hash,
            commit_message: None,
        })
        .collect();

    display_file_history(f, limit, layout, history)
}

fn display_file_history(
    f: &str,
    limit: usize,
    layout: &Layout,
    history: Vec<SnapshotInfo>,
) -> Result<()> {
    let clean_path = if f.starts_with(".\\") {
        &f[2..]
    } else if f.starts_with("./") {
        &f[2..]
    } else {
        f
    };

    layout.header_dashboard("FILE HISTORY");
    layout.section_branch("fi", clean_path);

    if history.is_empty() {
        layout.warning("No history found.");
    } else {
        layout.legend(&[
            ("● Is Latest", ""),
            ("· Past", ""),
            ("A", "Added"),
            ("M", "Modified"),
            ("D", "Deleted"),
        ]);
        println!();

        for (i, snap) in history.iter().take(limit).enumerate() {
            let hash_short = if snap.content_hash.len() >= 8 {
                &snap.content_hash[..8]
            } else {
                &snap.content_hash
            };

            let ts_string = snap.timestamp.to_string();
            let timestamp_parts: Vec<&str> = ts_string.split('T').collect();
            let time_only = if timestamp_parts.len() > 1 {
                let time_parts: Vec<&str> = timestamp_parts[1].split('.').collect();
                time_parts[0]
            } else {
                ""
            };

            layout.row_history_compact(hash_short, "M", clean_path, time_only, i == 0, None);
        }
        layout.footer_pagination(history.len().min(limit), history.len(), limit);
    }
    layout.section_end();
    layout.footer("Shift+Click the hash or 'mnem open' to view in your IDE.");
    Ok(())
}

// Try to use daemon for dashboard view
fn try_daemon_dashboard_view(
    limit: usize,
    layout: &Layout,
    project_path: &std::path::Path,
) -> Result<()> {
    let mut client = DaemonClient::connect()?;

    let res = client.call(
        methods::PROJECT_GET_ACTIVITY,
        serde_json::json!({
            "limit": limit,
            "project_path": project_path.to_string_lossy().to_string()
        }),
    )?;

    let history: Vec<SnapshotInfo> = serde_json::from_value(res)?;

    // Get project name from tracked file
    let tracked_file = project_path.join(".mnemosyne").join("tracked");
    let project_name = if let Ok(content) = std::fs::read_to_string(&tracked_file) {
        content
            .lines()
            .find(|l| l.starts_with("project_name:"))
            .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        "Unknown".to_string()
    };

    display_dashboard_view(limit, layout, project_path, project_name, history)
}

// Fallback: direct database access for dashboard view
fn handle_dashboard_view_direct(
    limit: usize,
    layout: &Layout,
    project_path: &std::path::Path,
) -> Result<()> {
    let base_dir = get_base_dir()?;
    let repo = Repository::open(base_dir, project_path.to_path_buf())?;

    let history_db = repo.get_recent_activity(limit)?;

    // Convert to SnapshotInfo format
    let history: Vec<SnapshotInfo> = history_db
        .into_iter()
        .map(|sn| SnapshotInfo {
            id: sn.id,
            file_path: sn.file_path,
            timestamp: sn.timestamp,
            content_hash: sn.content_hash,
            git_branch: sn.git_branch,
            commit_hash: sn.commit_hash,
            commit_message: None,
        })
        .collect();

    let project_name = repo.project.name.clone();
    display_dashboard_view(limit, layout, project_path, project_name, history)
}

fn display_dashboard_view(
    limit: usize,
    layout: &Layout,
    project_path: &std::path::Path,
    project_name: String,
    history: Vec<SnapshotInfo>,
) -> Result<()> {
    let mut by_branch: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for snap in &history {
        let branch = snap
            .git_branch
            .clone()
            .unwrap_or_else(|| "main".to_string());
        by_branch.entry(branch).or_default().push(snap);
    }

    layout.header_dashboard(&format!("HISTORY: {}", project_name));

    layout.section_branch("rc", "Recent Activity");
    if history.is_empty() {
        layout.item_simple("No recent activity");
    } else {
        for (i, snap) in history.iter().take(10).enumerate() {
            let hash_short = if snap.content_hash.len() >= 8 {
                &snap.content_hash[..8]
            } else {
                &snap.content_hash
            };

            let ts_string = snap.timestamp.to_string();
            let timestamp_parts: Vec<&str> = ts_string.split('T').collect();
            let time_only = if timestamp_parts.len() > 1 {
                let time_parts: Vec<&str> = timestamp_parts[1].split('.').collect();
                time_parts[0]
            } else {
                ""
            };

            let p = snap.file_path.replace("\\\\?\\", "");
            let p_path = std::path::Path::new(&p);
            let display_path = if let Ok(rel) = p_path.strip_prefix(project_path) {
                rel.to_string_lossy().to_string()
            } else {
                p
            };

            layout.row_history_compact(hash_short, "M", &display_path, time_only, i == 0, None);
        }
    }
    layout.section_end();

    layout.legend(&[
        ("● Latest", ""),
        ("· Past", ""),
        ("M", "Mod"),
        ("C", "Checkpt"),
        ("G", "Commit"),
    ]);

    layout.footer("Shift+Click the hash or 'mnem open' to view in your IDE.");
    layout.footer_pagination(history.len().min(limit), history.len(), limit);

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
