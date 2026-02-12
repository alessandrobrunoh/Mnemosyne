use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{
    client::DaemonClient, env::get_base_dir, protocol::methods, storage::registry::ProjectRegistry,
    Repository,
};
use std::env;
use std::path::Path;

use crate::ui::{self, ButlerLayout};
use crate::utils;

pub fn list(args: &[String]) -> Result<()> {
    let common_args = utils::parse_common_args(args);

    let base_dir = get_base_dir()?;
    let registry = ProjectRegistry::new(&base_dir)?;
    let mut projects = registry.list_projects();
    projects.retain(|p| !p.name.is_empty() && p.name != "unknown" && p.path != "/");

    let cwd = env::current_dir()?;

    if projects.is_empty() {
        println!(
            "  {}",
            "No projects tracked yet. Run `mnem` in a project directory to start.".dark_grey()
        );
        return Ok(());
    }

    let total_count = projects.len();
    // Apply pagination
    projects.truncate(common_args.limit);

    ButlerLayout::header("TRACKED PROJECTS");
    ButlerLayout::section_start("pr", "Applied Projects");

    for (idx, p) in projects.iter().enumerate() {
        let id_tag = format!("{:02}", idx);
        let project_id_short = &p.id[..8];

        let project_name = Path::new(&p.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let p_path = Path::new(&p.path);
        let is_current = cwd.starts_with(p_path) || p_path.starts_with(&cwd);
        let bullet = if is_current { "●" } else { "•" };

        let file_url = format!("file://{}", p.path);
        let interactive_id = format!(
            "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
            file_url, project_id_short
        );
        let dot = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", file_url, bullet);

        let content = format!(
            "{}  {}  {}",
            if is_current {
                dot.green()
            } else {
                dot.dark_grey()
            },
            project_name.bold().white(),
            interactive_id.cyan(),
        );
        ButlerLayout::row_list(&id_tag, &content);
    }
    ButlerLayout::section_end();

    if total_count > projects.len() {
        ButlerLayout::item_simple(&format!(
            "... and {} more projects. Use --limit <n> to see all.",
            total_count - projects.len()
        ));
    }

    ButlerLayout::footer("Click a project ID to open it in your OS or type 'mnem <id>' for info.");
    Ok(())
}

pub fn project_info(args: &[String]) -> Result<()> {
    // Handle both 'mnem project <id>' and 'mnem <id>'
    let id_query = if args[1] == "project" {
        args.get(2)
            .ok_or_else(|| anyhow::anyhow!("Project ID is required"))?
    } else {
        &args[1]
    };

    let base_dir = get_base_dir()?;
    let registry = ProjectRegistry::new(&base_dir)?;
    let projects = registry.list_projects();

    let p = projects
        .iter()
        .find(|p| p.id.starts_with(id_query) || p.id == *id_query)
        .ok_or_else(|| anyhow::anyhow!("Project not found with ID: {}", id_query))?;

    let repo = Repository::open(base_dir, p.path.clone().into())?;
    let size = repo.get_project_size()?;
    let files = repo.list_files(None, None)?;
    let branches = repo.list_branches()?;
    let checkpoints = repo.list_checkpoints().unwrap_or_default();

    ButlerLayout::header("PROJECT INTELLIGENCE");
    ButlerLayout::section_start("inf", "General Information");

    let name = Path::new(&p.path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    ButlerLayout::item_simple(&format!("{}: {}", "Name".dark_grey(), name.bold().white()));
    ButlerLayout::item_simple(&format!("{}: {}", "ID".dark_grey(), p.id.clone().cyan()));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Root".dark_grey(),
        p.path.clone().white()
    ));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Size".dark_grey(),
        format!("{:.2} MB", size as f64 / 1024.0 / 1024.0).yellow()
    ));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Last Open".dark_grey(),
        p.last_open.clone().white()
    ));

    ButlerLayout::section_start("st", "Statistics");
    ButlerLayout::item_simple(&format!(
        "{} tracked files",
        files.len().to_string().green()
    ));
    ButlerLayout::item_simple(&format!(
        "{} active branches",
        branches.len().to_string().blue()
    ));
    ButlerLayout::item_simple(&format!(
        "{} checkpoints",
        checkpoints.len().to_string().yellow()
    ));

    ButlerLayout::section_end();
    Ok(())
}

pub fn forget(args: &[String]) -> Result<()> {
    let id_query = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("Project ID or path is required"))?;
    let prune = args.iter().any(|a| a == "--prune" || a == "--nuke");

    let home = get_base_dir()?;
    let mut registry = ProjectRegistry::new(&home)?;
    let projects = registry.list_projects();

    // Find the project by ID or path
    let project = projects
        .iter()
        .find(|p| p.id.starts_with(id_query) || p.id == *id_query || p.path == *id_query)
        .ok_or_else(|| anyhow::anyhow!("Project not found: {}", id_query))?;

    let project_id = project.id.clone();
    let project_name = project.name.clone();

    ButlerLayout::header("FORGET PROJECT");
    ButlerLayout::section_start("fg", "Removal Action");

    // 1. Unwatch if daemon is running
    if let Ok(mut client) = DaemonClient::connect() {
        let _ = client.call(
            methods::PROJECT_UNWATCH,
            serde_json::json!({"project_path": project.path}),
        );
    }

    // 2. Remove from registry
    registry.remove(&project_id)?;
    ButlerLayout::item_simple(&format!(
        "Removed {} from registry.",
        project_name.bold().white()
    ));

    // 3. Prune data if requested
    if prune {
        let db_path = home.join("projects").join(format!("{}.sqlite", project_id));
        if db_path.exists() {
            std::fs::remove_file(&db_path)?;
            ButlerLayout::item_simple(&format!(
                "{} Deleted project database: {}",
                "⚠".red(),
                project_id.cyan()
            ));
        }
        ButlerLayout::item_simple(&format!("{} Associated history purged.", "✓".green()));
    } else {
        ButlerLayout::item_simple(&format!("{} History preserved in vault.", "ℹ".blue()));
    }

    ButlerLayout::section_end();
    ButlerLayout::footer("Use 'mnem list' to see remaining projects.");
    Ok(())
}

pub fn watch(args: &[String]) -> Result<()> {
    let mut target_path = env::current_dir()?;

    if let Some(pos) = args.iter().position(|a| a == "-p") {
        if let Some(p) = args.get(pos + 1) {
            target_path = std::path::PathBuf::from(p);
            if target_path.is_relative() {
                target_path = env::current_dir()?.join(target_path);
            }
        }
    }

    ButlerLayout::header("PROJECT WATCHER");
    ButlerLayout::section_start("wa", "Monitoring");

    // Register project in local registry so it appears in 'mnem list'
    let base_dir = get_base_dir()?;
    let mut registry = ProjectRegistry::new(&base_dir)?;
    let project = registry.get_or_create(&target_path)?;

    ButlerLayout::item_simple(&format!(
        "Target: {}",
        target_path.to_string_lossy().bold().white()
    ));
    ButlerLayout::item_simple(&format!("ID: {}", project.id.cyan()));

    // 1. Ensure daemon is running
    match mnem_core::client::ensure_daemon() {
        Ok(true) => ButlerLayout::item_simple(&format!("{} mnemd daemon started.", "✓".green())),
        Ok(false) => {
            ButlerLayout::item_simple(&format!("{} mnemd daemon is running.", "✓".green()))
        }
        Err(e) => return Err(anyhow::anyhow!("Failed to start daemon: {}", e)),
    }

    // 2. Register with daemon
    ButlerLayout::item_simple(&format!(
        "{} Registering project and starting background scan...",
        "→".cyan()
    ));

    let mut client = DaemonClient::connect()?;
    let _ = client.call(
        methods::PROJECT_WATCH,
        serde_json::json!({"project_path": target_path.to_string_lossy()}),
    )?;

    ButlerLayout::item_simple(&format!(
        "{} Daemon is now monitoring this path.",
        "✓".green()
    ));
    ButlerLayout::section_end();

    ButlerLayout::footer(
        "Run 'mnem' to see project activity or 'mnem tui' for the visual interface.",
    );
    Ok(())
}

pub fn history(args: &[String], filter_path: Option<String>) -> Result<()> {
    let mut target_filter = filter_path;
    let mut display_name = None;

    let common_args = utils::parse_common_args(args);

    let base_dir = get_base_dir()?;
    let registry = ProjectRegistry::new(&base_dir)?;
    let projects = registry.list_projects();

    // 1. Project ID Filtering & Auto-Watch
    if target_filter.is_none() && args.len() > 2 {
        let query = &args[2];
        if !query.starts_with('-') {
            if let Some(p) = projects
                .iter()
                .find(|p| p.id.starts_with(query) || p.id == *query)
            {
                target_filter = Some(p.path.clone());
                display_name = Some(p.name.clone());

                // Ensure daemon is watching this project before requesting activity
                let _ = mnem_core::client::ensure_daemon();
                if let Ok(mut client) = DaemonClient::connect() {
                    let _ = client.call(
                        methods::PROJECT_WATCH,
                        serde_json::json!({"project_path": p.path}),
                    );
                }
            }
        }
    }

    // Resolve display name if we have a path but no name yet
    if let Some(ref path) = target_filter {
        if display_name.is_none() {
            display_name = projects
                .iter()
                .find(|p| p.path == *path)
                .map(|p| p.name.clone());
        }
    }

    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::PROJECT_GET_ACTIVITY,
        serde_json::json!({
            "limit": common_args.limit,
            "project_path": target_filter,
            "branch": common_args.branch
        }),
    )?;

    let activity: Vec<mnem_core::protocol::SnapshotInfo> = serde_json::from_value(res)?;

    // Fetch checkpoints to integrate into activity
    let repo = Repository::init()?;
    let checkpoints = repo.list_checkpoints().unwrap_or_default();

    let title = match display_name {
        Some(name) => {
            if name.is_empty() || name == "unknown" {
                "PROJECT HISTORY".to_string()
            } else {
                format!("HISTORY: {}", name.to_uppercase())
            }
        }
        None => {
            if target_filter.is_some() {
                "PROJECT HISTORY".to_string()
            } else {
                "GLOBAL HISTORY".to_string()
            }
        }
    };
    ButlerLayout::header(&title);

    ButlerLayout::legend(&[
        ("C", "Created"),
        ("M", "Modified"),
        ("D", "Deleted"),
        ("🔖", "Checkpoint"),
        ("📦", "Commit"),
    ]);

    // Extract and group commits from activity
    use std::collections::HashMap;
    let mut commits_by_hash: HashMap<String, (String, String, String, usize)> = HashMap::new();

    for s in activity.iter() {
        if let Some(ref commit_hash) = s.commit_hash {
            if !commits_by_hash.contains_key(commit_hash) {
                let msg = s
                    .commit_message
                    .as_deref()
                    .unwrap_or("No message")
                    .to_string();
                let timestamp = chrono::DateTime::parse_from_rfc3339(&s.timestamp)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|_| s.timestamp[..16].to_string());
                commits_by_hash.insert(
                    commit_hash.clone(),
                    (commit_hash.clone(), msg, timestamp, 0),
                );
            }
            // Increment file count for this commit
            if let Some(entry) = commits_by_hash.get_mut(commit_hash) {
                entry.3 += 1;
            }
        }
    }

    let mut last_branch: Option<String> = None;
    let mut last_commit: Option<String> = None;

    // Show checkpoints section first
    if !checkpoints.is_empty() {
        ButlerLayout::section_start("🔖", "Project Checkpoints");
        for (hash, timestamp, description) in checkpoints.iter().take(5) {
            let hash_short = &hash[..8.min(hash.len())];
            let timestamp_formatted = chrono::DateTime::parse_from_rfc3339(timestamp)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| timestamp[..16].to_string());

            let desc = description.as_deref().unwrap_or("No description");
            let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
            let clickable_hash = ui::Hyperlink::action(&styled_hash, "checkpoint-info", hash);

            let content = format!("{}   {}", timestamp_formatted.dark_grey(), desc.white());
            ButlerLayout::row_snapshot(&clickable_hash, &content);
        }
        if checkpoints.len() > 5 {
            ButlerLayout::item_simple(
                &format!(
                    "... and {} more. Use 'mnem checkpoint-info <hash>' for details.",
                    checkpoints.len() - 5
                )
                .dark_grey()
                .to_string(),
            );
        }
        ButlerLayout::section_end();
    }

    // Show Git Commits section
    if !commits_by_hash.is_empty() {
        ButlerLayout::section_start("📦", "Git Commits");
        let mut commits: Vec<_> = commits_by_hash.values().cloned().collect();
        commits.sort_by(|a, b| b.2.cmp(&a.2)); // Sort by timestamp desc

        for (commit_hash, message, timestamp, file_count) in commits.iter().take(10) {
            let hash_short = &commit_hash[..7.min(commit_hash.len())];
            let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
            let clickable_hash = ui::Hyperlink::action(&styled_hash, "open", commit_hash);

            let content = format!(
                "{}  {}  {}",
                timestamp.clone().dark_grey(),
                format!("{} files", file_count).white(),
                message.clone().white()
            );
            ButlerLayout::row_snapshot(&clickable_hash, &content);
        }
        if commits.len() > 10 {
            ButlerLayout::item_simple(
                &format!("... and {} more commits.", commits.len() - 10)
                    .dark_grey()
                    .to_string(),
            );
        }
        ButlerLayout::section_end();
    }

    // Apply display pagination: N items PER branch
    let paginated_activity =
        utils::paginate_per_branch(activity, common_args.limit, |s| s.git_branch.clone());

    for s in paginated_activity.iter() {
        let hash_short = &s.content_hash[..7];
        let branch = s.git_branch.as_deref().unwrap_or("main");

        // --- GROUP BY BRANCH (GitButler Style) ---
        if last_branch.as_ref().map(|s| s.as_str()) != Some(branch) {
            if last_branch.is_some() {
                ButlerLayout::section_end();
            }
            let tag = if branch.len() >= 2 {
                &branch[..2]
            } else {
                branch
            };
            ButlerLayout::section_start(tag, branch);
            last_branch = Some(branch.to_string());
            last_commit = None;
        }

        if let Some(ref commit_hash) = s.commit_hash {
            if last_commit.as_ref() != Some(commit_hash) {
                let msg = s.commit_message.as_deref().unwrap_or("No message");
                println!(
                    "┊   {} {}: \"{}\" [{}]",
                    "📦".yellow(),
                    "Commit".bold(),
                    msg.italic().white(),
                    commit_hash[..7].cyan()
                );
                last_commit = Some(commit_hash.clone());
            }
        }

        let label = match s.id % 3 {
            // Heuristic
            0 => "M".blue(),
            1 => "C".green(),
            _ => "D".red(),
        };

        let rel_path = utils::make_path_relative(&s.file_path);

        // The Hero: Clickable Hash
        let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
        let clickable_hash = ui::Hyperlink::action(&styled_hash, "open", &s.content_hash);

        let content = format!(
            "{}  {: <25}  {}",
            label,
            rel_path.white(),
            s.timestamp[11..16].dark_grey()
        );
        ButlerLayout::row_snapshot(&clickable_hash, &content);
    }

    if last_branch.is_some() {
        ButlerLayout::section_end();
    }

    if paginated_activity.len() >= common_args.limit {
        ButlerLayout::item_simple(&format!(
            "... showing first {}. Use --limit <n> to see more.",
            common_args.limit
        ));
    }
    ButlerLayout::footer("Shift+Click the hash to open a snapshot in your IDE.");
    Ok(())
}

pub fn checkpoint(args: &[String]) -> Result<()> {
    let repo = Repository::init()?;
    let desc = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "Manual Checkpoint".into());

    let hash = repo.create_checkpoint(Some(&desc))?;

    ButlerLayout::header("CHECKPOINT");
    ButlerLayout::section_start("cp", "Semantic Snapshot");
    ButlerLayout::item_simple(&format!(
        "{} Checkpoint saved: {}",
        "√".green(),
        desc.bold().white()
    ));
    ButlerLayout::item_simple(&format!("{} Hash: {}", "→".cyan(), hash.cyan().bold()));
    ButlerLayout::section_end();
    Ok(())
}

pub fn list_checkpoints(_args: &[String]) -> Result<()> {
    let repo = Repository::init()?;
    let checkpoints = repo.list_checkpoints()?;

    ButlerLayout::header("PROJECT CHECKPOINTS");
    ButlerLayout::section_start("cp", "Snapshots History");

    if checkpoints.is_empty() {
        ButlerLayout::item_simple(&"No checkpoints found.".dark_grey().to_string());
    }

    for (hash, timestamp, description) in checkpoints {
        let hash_short = &hash[..8];
        let timestamp_styled = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or(timestamp);

        let desc = description.unwrap_or_else(|| "No description".into());

        let content = format!(
            "{}  {}  {}",
            hash_short.cyan().bold(),
            timestamp_styled.dark_grey(),
            desc.white()
        );
        ButlerLayout::row_list("●", &content);
    }

    ButlerLayout::section_end();
    ButlerLayout::footer("Use 'mnem restore --checkpoint <hash>' to revert everything.");
    Ok(())
}

pub fn checkpoint_info(args: &[String]) -> Result<()> {
    let hash_query = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("Usage: mnem checkpoint-info <hash>"))?;

    let repo = Repository::init()?;
    let (timestamp, file_states_json) = repo
        .db
        .get_checkpoint_by_hash(hash_query)?
        .ok_or_else(|| anyhow::anyhow!("Checkpoint not found: {}", hash_query))?;

    let file_states: Vec<(String, String)> = serde_json::from_str(&file_states_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse checkpoint data: {}", e))?;

    let timestamp_formatted = chrono::DateTime::parse_from_rfc3339(&timestamp)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or(timestamp);

    let hash_short = &hash_query[..8.min(hash_query.len())];

    ButlerLayout::header("CHECKPOINT DETAILS");
    ButlerLayout::section_start("cp", &format!("Checkpoint {}", hash_short.cyan().bold()));

    ButlerLayout::item_simple(&format!(
        "{} Timestamp: {}",
        "●".white(),
        timestamp_formatted.white()
    ));
    ButlerLayout::item_simple(&format!(
        "{} Files: {}",
        "●".white(),
        file_states.len().to_string().white()
    ));
    ButlerLayout::item_simple(&format!("{} Hash: {}", "●".white(), hash_short));

    println!();
    ButlerLayout::section_start("fi", "File States");

    for (file_path, content_hash) in &file_states {
        let rel_path = utils::make_path_relative(file_path);
        let hash_short = &content_hash[..7.min(content_hash.len())];

        let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
        let clickable_hash = ui::Hyperlink::action(&styled_hash, "open", content_hash);

        let content = format!("{:<50}", rel_path.white());
        ButlerLayout::row_snapshot(&clickable_hash, &content);
    }

    ButlerLayout::section_end();
    ButlerLayout::footer(
        "Click hash to view file content. Use 'mnem restore --checkpoint <hash>' to revert.",
    );
    Ok(())
}

pub fn commits(args: &[String]) -> Result<()> {
    let common_args = utils::parse_common_args(args);
    let repo = Repository::init()?;
    let commits = repo.list_commits()?;

    ButlerLayout::header("GIT COMMITS");
    ButlerLayout::section_start("gc", "All Commits");

    if commits.is_empty() {
        ButlerLayout::item_simple(&"No commits found.".dark_grey().to_string());
        ButlerLayout::section_end();
        return Ok(());
    }

    // Apply pagination
    let paginated = commits
        .into_iter()
        .take(common_args.limit)
        .collect::<Vec<_>>();

    for (commit_hash, message, author, timestamp, file_count) in paginated {
        let hash_short = &commit_hash[..7.min(commit_hash.len())];
        let timestamp_formatted = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|_| timestamp[..16].to_string());

        let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
        let clickable_hash = ui::Hyperlink::action(&styled_hash, "commit-info", &commit_hash);

        let content = format!(
            "{}  {}  {}  {}",
            timestamp_formatted.dark_grey(),
            format!("{} files", file_count).white(),
            author.white(),
            message.white()
        );
        ButlerLayout::row_snapshot(&clickable_hash, &content);
    }

    ButlerLayout::section_end();
    ButlerLayout::footer(
        "Use 'mnem commit-info <hash>' for details or 'mnem log-commits' to see files.",
    );
    Ok(())
}

pub fn commit_info(args: &[String]) -> Result<()> {
    let hash_query = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("Usage: mnem commit-info <hash>"))?;

    let repo = Repository::init()?;
    let (_commit_hash, message, author, timestamp) = repo
        .get_commit_details(hash_query)?
        .ok_or_else(|| anyhow::anyhow!("Commit not found: {}", hash_query))?;

    let files = repo.get_commit_files(hash_query)?;

    let timestamp_formatted = chrono::DateTime::parse_from_rfc3339(&timestamp)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or(timestamp);

    let hash_short = &hash_query[..7.min(hash_query.len())];

    ButlerLayout::header("COMMIT DETAILS");
    ButlerLayout::section_start("gc", &format!("Commit {}", hash_short.cyan().bold()));

    ButlerLayout::item_simple(&format!("{} Author: {}", "●".white(), author.white()));
    ButlerLayout::item_simple(&format!(
        "{} Date: {}",
        "●".white(),
        timestamp_formatted.white()
    ));
    ButlerLayout::item_simple(&format!("{} Message: {}", "●".white(), message.white()));
    ButlerLayout::item_simple(&format!(
        "{} Files: {}",
        "●".white(),
        files.len().to_string().white()
    ));
    ButlerLayout::item_simple(&format!("{} Hash: {}", "●".white(), hash_short));

    println!();
    ButlerLayout::section_start("fi", "Modified Files");

    for (file_path, content_hash, file_timestamp) in files {
        let rel_path = utils::make_path_relative(&file_path);
        let hash_short = &content_hash[..7.min(content_hash.len())];

        let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
        let clickable_hash = ui::Hyperlink::action(&styled_hash, "open", &content_hash);

        let file_time = chrono::DateTime::parse_from_rfc3339(&file_timestamp)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|_| file_timestamp[11..19].to_string());

        let content = format!("{:<50}  {}", rel_path.white(), file_time.dark_grey());
        ButlerLayout::row_snapshot(&clickable_hash, &content);
    }

    ButlerLayout::section_end();
    ButlerLayout::footer("Click hash to view file content.");
    Ok(())
}

pub fn log_commits(args: &[String]) -> Result<()> {
    let common_args = utils::parse_common_args(args);
    let repo = Repository::init()?;
    let commits = repo.list_commits()?;

    ButlerLayout::header("GIT COMMITS LOG");
    ButlerLayout::section_start("gl", "Commit History");

    if commits.is_empty() {
        ButlerLayout::item_simple(&"No commits found.".dark_grey().to_string());
        ButlerLayout::section_end();
        return Ok(());
    }

    let paginated = commits
        .into_iter()
        .take(common_args.limit)
        .collect::<Vec<_>>();

    for (commit_hash, message, author, timestamp, file_count) in paginated {
        let hash_short = &commit_hash[..7.min(commit_hash.len())];
        let timestamp_formatted = chrono::DateTime::parse_from_rfc3339(&timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|_| timestamp[..16].to_string());

        ButlerLayout::item_simple(&format!(
            "{} {} {} {}",
            "┌".dark_grey(),
            hash_short.cyan().bold(),
            timestamp_formatted.dark_grey(),
            format!("({} files)", file_count).white()
        ));
        ButlerLayout::item_simple(&format!(
            "{} {} {}",
            "│".dark_grey(),
            "👤".yellow(),
            author.white()
        ));
        ButlerLayout::item_simple(&format!(
            "{} {} {}",
            "│".dark_grey(),
            "💬".blue(),
            message.white()
        ));

        // Show files in this commit
        if let Ok(files) = repo.get_commit_files(&commit_hash) {
            for (file_path, content_hash, _) in files.iter().take(10) {
                let rel_path = utils::make_path_relative(file_path);
                let hash_short = &content_hash[..7.min(content_hash.len())];

                let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
                let clickable_hash = ui::Hyperlink::action(&styled_hash, "open", content_hash);

                ButlerLayout::item_simple(&format!("{}   {}", "│".dark_grey(), rel_path.white()));
                ButlerLayout::item_simple(&format!(
                    "{}   {}",
                    "│".dark_grey(),
                    clickable_hash.to_string()
                ));
            }
            if files.len() > 10 {
                ButlerLayout::item_simple(&format!(
                    "{}   ... and {} more files",
                    "│".dark_grey(),
                    files.len() - 10
                ));
            }
        }
        ButlerLayout::item_simple(&format!("{}", "└".dark_grey()));
        println!();
    }

    ButlerLayout::section_end();
    ButlerLayout::footer("Use 'mnem commit-info <hash>' for full details.");
    Ok(())
}

pub fn delete_checkpoint(args: &[String]) -> Result<()> {
    let hash_query = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("Usage: mnem delete-checkpoint <hash>"))?;

    let repo = Repository::init()?;
    let deleted = repo.delete_checkpoint(hash_query)?;

    if deleted {
        ButlerLayout::header("CHECKPOINT DELETED");
        ButlerLayout::section_start("cp", "Removal");
        ButlerLayout::item_simple(&format!(
            "{} Checkpoint {} deleted successfully.",
            "√".green(),
            hash_query.clone().cyan().bold()
        ));
        ButlerLayout::section_end();
        Ok(())
    } else {
        Err(anyhow::anyhow!("Checkpoint not found: {}", hash_query))
    }
}

pub fn git_event(args: &[String]) -> Result<()> {
    if args.len() < 5 {
        return Err(anyhow::anyhow!(
            "Usage: mnem git-event <hash> <message> <author> <timestamp>"
        ));
    }
    let hash = &args[2];
    let message = &args[3];
    let author = &args[4];
    let timestamp = &args[5];

    let repo = Repository::init()?;
    repo.db
        .insert_git_commit(hash, message, author, timestamp)?;

    let history = repo.db.get_recent_activity(50)?;
    for s in history {
        if s.commit_hash.is_none() {
            repo.db.link_snapshot_to_commit(s.id, hash)?;
        }
    }

    Ok(())
}

pub fn git_hook() -> Result<()> {
    let cwd = env::current_dir()?;
    crate::git::install_hook(&cwd)?;

    ButlerLayout::header("GIT INTEGRATION");
    ButlerLayout::section_start("gh", "Post-Commit Hook");
    ButlerLayout::item_simple(&format!(
        "{} Installed to .git/hooks/post-commit",
        "√".green()
    ));
    ButlerLayout::section_end();
    Ok(())
}

pub fn statistics(args: &[String]) -> Result<()> {
    let _common_args = utils::parse_common_args(args);
    let mut client = DaemonClient::connect()?;

    // 1. Resolve project context (similar to activity)
    let base_dir = get_base_dir()?;
    let registry = ProjectRegistry::new(&base_dir)?;
    let projects = registry.list_projects();
    let mut project_path = None;
    let mut display_name = None;

    if args.len() > 2 {
        let query = &args[2];
        if !query.starts_with('-') {
            if let Some(p) = projects
                .iter()
                .find(|p| p.id.starts_with(query) || p.id == *query)
            {
                project_path = Some(p.path.clone());
                display_name = Some(p.name.clone());
            }
        }
    }

    // Auto-watch if in project dir
    if project_path.is_none() {
        if let Ok(cwd) = env::current_dir() {
            if let Some(p) = projects.into_iter().find(|p| {
                let p_path = Path::new(&p.path);
                cwd == p_path || cwd.starts_with(p_path)
            }) {
                project_path = Some(p.path);
                display_name = Some(p.name);
            }
        }
    }

    let res = client.call(
        methods::PROJECT_GET_STATISTICS,
        serde_json::json!({ "project_path": project_path }),
    )?;

    let stats: mnem_core::protocol::ProjectStatisticsResponse = serde_json::from_value(res)?;

    let title = match display_name {
        Some(name) => {
            if name.is_empty() || name == "unknown" {
                "PROJECT STATISTICS".to_string()
            } else {
                format!("STATISTICS: {}", name.to_uppercase())
            }
        }
        None => "GLOBAL STATISTICS".to_string(),
    };
    ButlerLayout::header(&title);

    // --- Key Metrics ---
    ButlerLayout::section_start("me", "Key Metrics");
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Total Snapshots".dark_grey(),
        stats.total_snapshots.to_string().green().bold()
    ));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Tracked Files".dark_grey(),
        format!("{} files", stats.total_files).white()
    ));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Active Branches".dark_grey(),
        format!("{} branches", stats.total_branches).blue()
    ));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Git Commits".dark_grey(),
        format!("{} commits", stats.total_commits).yellow()
    ));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Vault Size".dark_grey(),
        format!("{:.2} MB", stats.size_bytes as f64 / 1024.0 / 1024.0).cyan()
    ));
    ButlerLayout::item_simple(&format!(
        "{}: {}",
        "Last Activity".dark_grey(),
        stats.last_activity.white()
    ));

    // --- Activity by Day (Mini Chart) ---
    if !stats.activity_by_day.is_empty() {
        ButlerLayout::section_start("ad", "Activity by Day (Last 30 Days)");
        let max_val = stats
            .activity_by_day
            .iter()
            .map(|(_, v)| *v)
            .max()
            .unwrap_or(1);
        for (day, count) in stats.activity_by_day.iter().take(7) {
            let bar_len = if max_val > 0 {
                (count * 20 / max_val).max(1)
            } else {
                0
            };
            let bar = "█".repeat(bar_len);
            ButlerLayout::item_simple(&format!(
                "{: <10} | {} {}",
                day.as_str().dark_grey(),
                bar.with(ui::ACCENT),
                count.to_string().white()
            ));
        }
    }

    // --- Activity by Hour ---
    if !stats.activity_by_hour.is_empty() {
        ButlerLayout::section_start("ah", "Activity by Hour (24h Distribution)");
        let max_val = stats
            .activity_by_hour
            .iter()
            .map(|(_, v)| *v)
            .max()
            .unwrap_or(1);
        let mut hours_map = std::collections::HashMap::new();
        for (h, c) in stats.activity_by_hour {
            hours_map.insert(h, c);
        }

        let mut line = String::new();
        for h in 0..24 {
            let count = hours_map.get(&h).cloned().unwrap_or(0);
            let intensity = if max_val > 0 {
                (count * 7 / max_val).min(7)
            } else {
                0
            };

            // Using block elements:  ▂▃▄▅▆▇█
            let symbol = match intensity {
                0 => " ".dark_grey().to_string(),
                1 => "▂".dark_grey().to_string(),
                2 => "▃".white().to_string(),
                3 => "▄".white().to_string(),
                4 => "▅".cyan().to_string(),
                5 => "▆".cyan().to_string(),
                6 => "▇".blue().to_string(),
                _ => "█".blue().bold().to_string(),
            };
            line.push_str(&symbol);
        }

        // Add a scale for easier reading
        ButlerLayout::item_simple(&format!("    {}", line));
        ButlerLayout::item_simple(&format!(
            "    {}{}{}{}{}",
            "00".dark_grey(),
            "      06".dark_grey(),
            "      12".dark_grey(),
            "      18".dark_grey(),
            "    23".dark_grey()
        ));
    }

    // --- Top Files ---
    if !stats.top_files.is_empty() {
        ButlerLayout::section_start("tf", "Most Active Files");
        for (path, count) in stats.top_files.iter().take(5) {
            let rel_path = utils::make_path_relative(path);
            ButlerLayout::item_simple(&format!(
                "{: <30} | {}",
                rel_path.white(),
                count.to_string().green()
            ));
        }
    }

    // --- Extension Distribution ---
    if !stats.extensions.is_empty() {
        ButlerLayout::section_start("ex", "Language Distribution");
        let max_val = stats.extensions.iter().map(|(_, v)| *v).max().unwrap_or(1);
        for (ext, count) in stats.extensions.iter().take(7) {
            let bar_len = if max_val > 0 {
                (count * 15 / max_val).max(1)
            } else {
                0
            };
            let bar = "▒".repeat(bar_len);
            ButlerLayout::item_simple(&format!(
                "{: <10} | {} {}",
                ext.as_str().yellow(),
                bar.dark_grey(),
                count.to_string().white()
            ));
        }
    }

    ButlerLayout::section_end();
    ButlerLayout::footer("History captured by Mnemosyne. Keep building!");
    Ok(())
}
