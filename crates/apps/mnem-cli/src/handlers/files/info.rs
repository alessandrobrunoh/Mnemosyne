use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::ui::{Layout, Presentable};
use mnem_core::client::DaemonClient;
use mnem_core::protocol::methods;

#[derive(Serialize)]
pub struct ProjectInfoResponse {
    pub success: bool,
    pub name: String,
    pub path: String,
    pub id: String,
    pub size_bytes: u64,
    pub total_snapshots: usize,
    pub total_files: usize,
    pub total_branches: usize,
    pub extensions: HashMap<String, usize>,
    pub source: String,
}

impl Presentable for ProjectInfoResponse {
    fn render_tui(&self) -> Result<()> {
        let layout = Layout::new();
        layout.header_dashboard("PROJECT INFO");
        layout.section_branch("pr", &self.name);
        layout.row_labeled("◫", "Path", &self.path);
        layout.row_labeled("◆", "ID", &self.id);
        layout.row_metric(
            "",
            "Size",
            &format!("{:.2} MB", self.size_bytes as f64 / 1024.0 / 1024.0),
        );
        layout.section_end();

        layout.section_branch("st", "Activity Summary");
        layout.row_metric("", "Total Snapshots", &self.total_snapshots.to_string());
        layout.row_metric("", "Unique Files", &self.total_files.to_string());
        layout.row_metric("", "Branches", &self.total_branches.to_string());
        layout.section_end();

        if !self.extensions.is_empty() {
            layout.section_branch("fi", "File Types");
            let mut top_types: Vec<_> = self.extensions.iter().collect();
            top_types.sort_by(|a, b| b.1.cmp(a.1));

            for (ext, count) in top_types.iter().take(6) {
                let icon = match ext.as_str() {
                    "rs" => "🦀",
                    "js" | "ts" | "jsx" | "tsx" => "📜",
                    "py" => "🐍",
                    "go" => "🐹",
                    "java" => "☕",
                    "c" | "cpp" | "h" | "hpp" => "⚙️",
                    "html" | "css" | "scss" | "sass" => "🌐",
                    "json" | "toml" | "yaml" | "yml" => "📝",
                    "md" | "markdown" => "📖",
                    "txt" => "📄",
                    _ => "📄",
                };
                layout.row_key_value(
                    &format!("{} .{}", icon, ext),
                    &format!("{} files", count),
                );
            }
            layout.section_end();
        }

        layout.empty();
        layout.badge_success("OK", &format!("Project loaded from {}", self.source));
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_info(_project: Option<String>, json: bool) -> Result<()> {
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::Repository;

    let base_dir = get_base_dir()?;
    let cwd = std::env::current_dir()?;
    let tracked_file = cwd.join(".mnemosyne").join("tracked");

    // Try daemon first
    let daemon = DaemonClient::connect().ok();

    if let Some(mut client) = daemon {
        // Daemon is running - use it to get project info
        match client.call(
            methods::PROJECT_GET_STATISTICS,
            serde_json::json!({ "project_path": cwd.to_string_lossy().to_string() }),
        ) {
            Ok(res) => {
                let stats: mnem_core::protocol::ProjectStatisticsResponse =
                    serde_json::from_value(res)?;

                let project_path = cwd.to_string_lossy().to_string();
                let project_name = std::path::Path::new(&project_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let response = ProjectInfoResponse {
                    success: true,
                    name: project_name,
                    path: project_path,
                    id: "tracked".to_string(),
                    size_bytes: stats.size_bytes,
                    total_snapshots: stats.total_snapshots,
                    total_files: stats.total_files,
                    total_branches: stats.total_branches,
                    extensions: stats.extensions.into_iter().collect(),
                    source: "daemon".to_string(),
                };

                if json {
                    println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
                } else {
                    response.render_tui()?;
                }
                return Ok(());
            }
            Err(e) => {
                let msg = e.to_string();
                if !msg.contains("lock") && !msg.contains("Database already open") {
                    // If it's not a lock error, we might want to report it
                }
            }
        }
    }

    // Try direct access (daemon not running or error)
    let repo = match Repository::open(base_dir.clone(), cwd.clone()) {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
            if json {
                println!("{}", serde_json::json!({
                    "success": false,
                    "error": msg,
                    "code": if msg.contains("lock") { "DB_LOCKED" } else if !tracked_file.exists() { "NOT_TRACKED" } else { "UNKNOWN_ERROR" }
                }));
            } else {
                let layout = Layout::new();
                if msg.contains("lock") || msg.contains("Database already open") {
                    layout.header_dashboard("PROJECT LOCKED");
                    layout.section_branch("pr", "Current Folder");
                    layout.row_labeled("◫", "Path", &cwd.to_string_lossy());
                    layout.section_end();
                    layout.empty();
                    layout.badge_error("ERROR", "Daemon is running");
                    layout.info_bright(
                        "Run 'mnem off' to access directly, or the daemon is actively tracking this project.",
                    );
                } else if !tracked_file.exists() {
                    layout.header_dashboard("PROJECT NOT TRACKED");
                    layout.section_branch("pr", "Current Folder");
                    layout.row_labeled("◫", "Path", &cwd.to_string_lossy());
                    layout.section_end();
                    layout.empty();
                    layout.badge_error("ERROR", "This project is not tracked");
                    layout.info_bright("Run 'mnem track' to start tracking this project.");
                } else {
                    layout.header_dashboard("PROJECT ERROR");
                    layout.section_branch("pr", "Current Folder");
                    layout.row_labeled("◫", "Path", &cwd.to_string_lossy());
                    layout.section_end();
                    layout.empty();
                    layout.badge_error("ERROR", &msg);
                }
            }
            return Ok(());
        }
    };

    let history = repo.get_recent_activity(1000)?;
    let files: Vec<_> = history.iter().map(|s| &s.file_path).collect();
    let unique_files: std::collections::HashSet<_> = files.iter().collect();
    let size = repo.get_project_size()?;

    let mut extensions: HashMap<String, usize> = HashMap::new();
    for f in &unique_files {
        let ext = std::path::Path::new(f)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("no ext");
        *extensions.entry(ext.to_string()).or_insert(0) += 1;
    }

    let mut branches: std::collections::HashSet<_> = std::collections::HashSet::new();
    for s in &history {
        if let Some(b) = &s.git_branch {
            branches.insert(b.clone());
        } else {
            branches.insert("main".to_string());
        }
    }

    let response = ProjectInfoResponse {
        success: true,
        name: repo.project.name.clone(),
        path: repo.project.path.clone(),
        id: repo.project.id.clone(),
        size_bytes: size,
        total_snapshots: history.len(),
        total_files: unique_files.len(),
        total_branches: branches.len(),
        extensions,
        source: "local".to_string(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}
