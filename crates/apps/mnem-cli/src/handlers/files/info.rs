use anyhow::Result;

use crate::ui::Layout;
use mnem_core::client::DaemonClient;
use mnem_core::protocol::methods;

pub fn handle_info(_project: Option<String>) -> Result<()> {
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::Repository;
    use std::collections::HashMap;

    let layout = Layout::new();
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

                // Get project info from daemon
                let project_path = cwd.to_string_lossy().to_string();
                let project_name = std::path::Path::new(&project_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                layout.header_dashboard("PROJECT INFO");
                layout.section_branch("pr", &project_name);
                layout.row_labeled("◫", "Path", &project_path);
                layout.row_labeled("◆", "ID", "tracked");
                layout.row_metric(
                    "",
                    "Size",
                    &format!("{:.2} MB", stats.size_bytes as f64 / 1024.0 / 1024.0),
                );
                layout.section_end();

                layout.section_branch("st", "Activity Summary");
                layout.row_metric("", "Total Snapshots", &stats.total_snapshots.to_string());
                layout.row_metric("", "Unique Files", &stats.total_files.to_string());
                layout.row_metric("", "Branches", &stats.total_branches.to_string());
                layout.section_end();

                if !stats.extensions.is_empty() {
                    layout.section_branch("fi", "File Types");
                    for (ext, count) in stats.extensions.iter().take(6) {
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
                layout.badge_success("OK", "Project loaded from daemon");
                return Ok(());
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("lock") || msg.contains("Database already open") {
                    // Fall through to try direct access
                } else {
                    // Some other daemon error, try direct access
                }
            }
        }
    }

    // Try direct access (daemon not running or error)
    let repo = match Repository::open(base_dir.clone(), cwd.clone()) {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
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
            return Ok(());
        }
    };

    let history = repo.get_recent_activity(1000)?;
    let files: Vec<_> = history.iter().map(|s| &s.file_path).collect();
    let unique_files: std::collections::HashSet<_> = files.iter().collect();
    let size = repo.get_project_size()?;

    let mut by_extension: HashMap<String, usize> = HashMap::new();
    for f in &unique_files {
        let ext = std::path::Path::new(f)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("no ext");
        *by_extension.entry(ext.to_string()).or_insert(0) += 1;
    }

    let mut top_types: Vec<_> = by_extension.into_iter().collect();
    top_types.sort_by(|a, b| b.1.cmp(&a.1));

    let mut branches: std::collections::HashSet<_> = std::collections::HashSet::new();
    for s in &history {
        if let Some(b) = &s.git_branch {
            branches.insert(b.clone());
        } else {
            branches.insert("main".to_string());
        }
    }

    layout.header_dashboard("PROJECT INFO");
    layout.section_branch("pr", &repo.project.name);

    layout.row_labeled("◫", "Path", &repo.project.path);
    layout.row_labeled("◆", "ID", &repo.project.id);
    layout.row_metric(
        "",
        "Size",
        &format!("{:.2} MB", size as f64 / 1024.0 / 1024.0),
    );

    layout.section_end();

    layout.section_branch("st", "Activity Summary");
    layout.row_metric("", "Total Snapshots", &history.len().to_string());
    layout.row_metric("", "Unique Files", &unique_files.len().to_string());
    layout.row_metric("", "Branches", &branches.len().to_string());

    layout.section_end();

    if !top_types.is_empty() {
        layout.section_branch("fi", "File Types");
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
            layout.row_key_value(&format!("{} .{}", icon, ext), &format!("{} files", count));
        }
        layout.section_end();
    }

    if let Ok(cps) = repo.list_checkpoints() {
        if !cps.is_empty() {
            layout.row_metric("", "Checkpoints", &cps.len().to_string());
        }
    }

    if let Ok(commits) = repo.list_commits() {
        if !commits.is_empty() {
            layout.row_metric("", "Git Commits", &commits.len().to_string());
        }
    }

    layout.empty();
    layout.badge_success("OK", "Project loaded successfully");

    Ok(())
}
