
use anyhow::Result;

use crate::ui::Layout;

pub fn handle_info(_project: Option<String>) -> Result<()> {
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::Repository;
    use std::collections::HashMap;

    let layout = Layout::new();
    let base_dir = get_base_dir()?;
    let cwd = std::env::current_dir()?;
    let repo = match Repository::open(base_dir.clone(), cwd.clone()) {
        Ok(r) => r,
        Err(_) => {
            layout.header_dashboard("PROJECT NOT TRACKED");
            layout.section_branch("pr", "Current Folder");
            layout.row_labeled("◫", "Path", &cwd.to_string_lossy());
            layout.section_end();
            layout.empty();
            layout.badge_error("ERROR", "This project is not tracked");
            layout.info_bright("Run 'mnem track' to start tracking this project.");
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
