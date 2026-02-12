use anyhow::Result;
use mnem_core::env::get_base_dir;
use mnem_core::Repository;
use std::env;
use std::path::Path;
use std::sync::Arc;

mod commands;
mod git;
mod i18n;
mod os;
mod theme;
mod ui;
mod utils;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Context detection
    let cwd = env::current_dir()?;
    let base_dir = get_base_dir()?;
    let registry = mnem_core::storage::registry::ProjectRegistry::new(&base_dir)?;
    let mut projects = registry.list_projects();
    projects.sort_by(|a, b| b.path.len().cmp(&a.path.len())); // Match longest path first (deepest)

    let tracked_project = projects.into_iter().find(|p| {
        let p_path = Path::new(&p.path);

        // --- PROTEZIONE ROOT ---
        // Se il progetto è la root o una cartella di sistema, ignoralo nell'auto-context
        if p.path == "/"
            || (p.path.starts_with("/Users") && p.path.split('/').count() <= 3)
            || p.name == "unknown"
        {
            return false;
        }

        cwd == p_path || cwd.starts_with(p_path)
    });

    if args.len() > 1 {
        let cmd = args[1].as_str();

        // Commands that require project context and daemon watching
        // Note: status and list are NOT included here because they handle their own connections
        let context_dependent = matches!(
            cmd,
            "log"
                | "diff"
                | "info"
                | "restore"
                | "checkpoint"
                | "tui"
                | "search"
                | "history"
                | "timeline"
                | "statistics"
                | "stats"
        );

        if context_dependent {
            if let Some(ref project) = tracked_project {
                let _ = mnem_core::client::ensure_daemon();
                if let Ok(mut client) = mnem_core::client::DaemonClient::connect() {
                    // DaemonClient::connect() now handles initialization
                    let _ = client.call(
                        mnem_core::protocol::methods::PROJECT_WATCH,
                        serde_json::json!({"project_path": project.path}),
                    );
                }
            }
        }

        let cmd_result = match cmd {
            "help" | "--help" | "-h" => Some(commands::general::help()),
            "version" | "--version" | "-v" => Some(commands::general::version()),
            "tui" => {
                let repo = Arc::new(Repository::init()?);
                Some(commands::tui::start(repo))
            }
            "start" => Some(commands::daemon::start(&args)),
            "stop" => Some(commands::daemon::stop()),
            "status" => Some(commands::daemon::status()),
            "log" => Some(commands::files::log(&args)),
            "diff" => Some(commands::files::diff(&args)),
            "info" => Some(commands::files::info(&args)),
            "open" | "o" => Some(commands::files::open(&args)),
            "cat" => Some(commands::files::cat(&args)),
            "restore" => Some(commands::files::restore(&args)),
            "search" | "grep" => Some(commands::files::search(&args)),
            "list" => Some(commands::workspace::list(&args)),
            "forget" => Some(commands::workspace::forget(&args)),
            "watch" => Some(commands::workspace::watch(&args)),
            "project" => Some(commands::workspace::project_info(&args)),
            "history" => Some(commands::workspace::history(&args, None)),
            "timeline" => Some(commands::files::timeline(&args)),
            "statistics" | "stats" => Some(commands::workspace::statistics(&args)),
            "checkpoint" => Some(commands::workspace::checkpoint(&args)),
            "checkpoints" => Some(commands::workspace::list_checkpoints(&args)),
            "checkpoint-info" => Some(commands::workspace::checkpoint_info(&args)),
            "commits" => Some(commands::workspace::commits(&args)),
            "commit-info" => Some(commands::workspace::commit_info(&args)),
            "log-commits" => Some(commands::workspace::log_commits(&args)),
            "git-event" => Some(commands::workspace::git_event(&args)),
            "git-hook" => Some(commands::workspace::git_hook()),
            "config" => Some(commands::maintenance::config(&args)),
            "setup-protocol" => Some(commands::maintenance::setup_protocol()),
            "gc" => Some(commands::maintenance::gc()),
            _ => {
                // Check if it's a project ID shortcut (hex string >= 4 chars)
                if args[1].len() >= 4 && args[1].chars().all(|c| c.is_ascii_hexdigit()) {
                    Some(commands::workspace::project_info(&args))
                } else {
                    eprintln!("Error: \"mnem {}\" is not a command. Type \"mnem --help\" to see all available commands.", args[1]);
                    std::process::exit(1);
                }
            }
        };

        if let Some(res) = cmd_result {
            return res;
        }
    }

    // Default behavior when running just `mnem`
    if let Some(project) = tracked_project {
        // Ensure daemon is running and watching this project
        let _ = mnem_core::client::ensure_daemon();
        if let Ok(mut client) = mnem_core::client::DaemonClient::connect() {
            // DaemonClient::connect() handles initialization
            let _ = client.call(
                mnem_core::protocol::methods::PROJECT_WATCH,
                serde_json::json!({"project_path": project.path}),
            );
        }

        // Show history for the project root
        commands::workspace::history(
            &["mnem".to_string(), "history".to_string()],
            Some(project.path),
        )
    } else {
        // Show project list
        commands::workspace::list(&args)
    }
}
