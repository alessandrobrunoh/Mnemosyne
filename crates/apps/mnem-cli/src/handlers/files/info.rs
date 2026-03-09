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
        let theme = layout.theme();

        layout.graph_branch_start(&format!("project: {}", self.name));
        
        // 1. Core Info
        layout.graph_node(
            &self.id,
            "ID",
            true,
            "tracked",
            None,
            theme.success_bright
        );
        layout.graph_node(
            &self.path,
            "PATH",
            false,
            "root",
            None,
            theme.text_dim
        );

        layout.graph_connector();

        // 2. Activity Summary
        layout.graph_block_header("📊", "activity", theme.timeline_purple);
        layout.graph_node(
            &self.total_snapshots.to_string(),
            "SNAPSHOTS",
            false,
            "total",
            None,
            theme.text_dim
        );
        layout.graph_node(
            &self.total_files.to_string(),
            "UNIQUE FILES",
            false,
            "total",
            None,
            theme.text_dim
        );
        layout.graph_node(
            &self.total_branches.to_string(),
            "BRANCHES",
            false,
            "total",
            None,
            theme.text_dim
        );

        layout.graph_connector();

        // 3. File Types
        if !self.extensions.is_empty() {
            layout.graph_block_header("📂", "file types", theme.timeline_cyan);
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
                layout.graph_node(
                    &format!("{} files", count),
                    &format!(".{}", ext),
                    false,
                    "count",
                    Some(icon),
                    theme.text_dim
                );
            }
        }

        layout.graph_branch_end();
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
                layout.graph_branch_start("project: Mnemosyne");
                if msg.contains("lock") || msg.contains("Database already open") {
                    layout.graph_node("LOCKED", "STATUS", false, "daemon running", None, crossterm::style::Color::Yellow);
                } else if !tracked_file.exists() {
                    layout.graph_node("NOT TRACKED", "STATUS", false, "untracked", None, crossterm::style::Color::Red);
                } else {
                    layout.graph_node("ERROR", "STATUS", false, &msg, None, crossterm::style::Color::Red);
                }
                layout.graph_branch_end();
            }
            return Ok(());
        }
    };

    let history = repo.get_recent_activity(1000)?;
    let unique_files: std::collections::HashSet<_> = history.iter().map(|s| &s.file_path).collect();
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
