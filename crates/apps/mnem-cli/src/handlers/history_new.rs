use crate::ui::Layout;
use anyhow::Result;
use mnem_core::storage::Repository;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn check_project_tracked(layout: &Layout) -> Result<(PathBuf, PathBuf, Repository)> {
    use mnem_core::env::get_base_dir;

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

    let repo = Repository::open(base_dir.clone(), cwd.clone())?;
    Ok((base_dir, cwd, repo))
}

pub fn handle_h(
    file: Option<String>,
    limit: Option<usize>,
    timeline: bool,
    _since: Option<String>,
    _branch: Option<String>,
) -> Result<()> {
    let layout = Layout::new();
    let (_, cwd, repo) = match check_project_tracked(&layout) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };

    let limit = limit.unwrap_or(20);

    if timeline {
        return handle_timeline_view(file, &layout);
    }

    if let Some(ref f) = file {
        return handle_file_history(f, limit, &layout, &repo);
    }

    handle_dashboard_view(limit, &layout, &repo, &cwd)
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

fn handle_file_history(f: &str, limit: usize, layout: &Layout, repo: &Repository) -> Result<()> {
    let clean_path = if f.starts_with(".\\") {
        &f[2..]
    } else if f.starts_with("./") {
        &f[2..]
    } else {
        f
    };
    let history = repo.get_history(clean_path)?;

    layout.header_dashboard("FILE HISTORY");
    layout.section_branch("fi", clean_path);

    if history.is_empty() {
        layout.warning("No history found.");
    } else {
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

            layout.row_history_compact(hash_short, "M", clean_path, time_only, i == 0);
        }
    }
    layout.section_end();
    layout.footer("Shift+Click the hash or 'mnem open' to view in your IDE.");
    Ok(())
}

fn handle_dashboard_view(
    limit: usize,
    layout: &Layout,
    repo: &Repository,
    cwd: &std::path::Path,
) -> Result<()> {
    let history = repo.get_recent_activity(limit)?;

    let mut by_branch: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for snap in &history {
        let branch = snap
            .git_branch
            .clone()
            .unwrap_or_else(|| "main".to_string());
        by_branch.entry(branch).or_default().push(snap);
    }

    layout.header_dashboard(&format!("HISTORY: {}", repo.project.name));

    layout.section_branch("cp", "Checkpoints");
    if let Ok(checkpoints) = repo.list_checkpoints() {
        if !checkpoints.is_empty() {
            for (hash, ts, desc) in checkpoints.iter().take(5) {
                let hash_short = &hash[..8.min(hash.len())];
                let timestamp_parts: Vec<&str> = ts.split('T').collect();
                let date_time = if timestamp_parts.len() > 1 {
                    let time_parts: Vec<&str> = timestamp_parts[1].split('.').collect();
                    format!("{} {}", timestamp_parts[0], time_parts[0])
                } else {
                    ts.to_string()
                };

                let msg = desc.as_deref().unwrap_or("No description");
                layout.row_history_compact(hash_short, "CP", msg, &date_time, false);
            }
        } else {
            layout.item_simple("No checkpoints");
        }
    }
    layout.section_end();

    layout.section_branch("git", "Git Commits");
    if let Ok(commits) = repo.list_commits() {
        if !commits.is_empty() {
            for (hash, author, msg, ts, files) in commits.iter().take(5) {
                let hash_short = &hash[..8.min(hash.len())];
                let timestamp_parts: Vec<&str> = ts.split('T').collect();
                let date_time = if timestamp_parts.len() > 1 {
                    let time_parts: Vec<&str> = timestamp_parts[1].split('.').collect();
                    format!("{} {}", timestamp_parts[0], time_parts[0])
                } else {
                    ts.to_string()
                };

                let desc = format!("{} files  {} - {}", files, author, msg);
                layout.row_history_compact(hash_short, "GIT", &desc, &date_time, false);
            }
        } else {
            layout.item_simple("No commits");
        }
    }
    layout.section_end();

    for (branch_name, snaps) in &by_branch {
        let branch_icon = if branch_name == "main" { "ma" } else { "br" };
        layout.section_branch(branch_icon, branch_name);

        for (i, snap) in snaps.iter().enumerate() {
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
            let display_path = if let Ok(rel) = p_path.strip_prefix(cwd) {
                rel.to_string_lossy().to_string()
            } else {
                p.to_string()
            };

            layout.row_history_compact(hash_short, "M", &display_path, time_only, i == 0);
        }
        layout.section_end();
    }

    layout.footer("Shift+Click the hash or 'mnem open' to view in your IDE.");

    Ok(())
}
