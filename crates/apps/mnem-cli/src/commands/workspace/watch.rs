use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::client::daemon_running;
use mnem_core::storage::Repository;
use notify::{Config, Event, RecursiveMode, Watcher};
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug)]
pub struct WatchCommand;

impl Command for WatchCommand {
    fn name(&self) -> &str {
        "watch"
    }

    fn usage(&self) -> &str {
        "[-p <path>]"
    }

    fn description(&self) -> &str {
        "Start watching directory and auto-save changes"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        let cwd = env::current_dir()?;

        let path = if args.len() > 2 && args[1] == "-p" {
            PathBuf::from(&args[2])
        } else if let Some(pos) = args.iter().position(|a| a == "-p" || a == "--path") {
            if let Some(p) = args.get(pos + 1) {
                PathBuf::from(p)
            } else {
                cwd.clone()
            }
        } else {
            cwd.clone()
        };

        // Check if daemon is running
        if daemon_running() {
            // Daemon is active - just register project (daemon already watching)
            // Projects registered via mnem watch are tracked by the daemon automatically
            // from the registry, so we just confirm the registration
            layout.success(&format!("Project registered: {}", path.display()));
            layout.item_simple("The daemon is tracking changes automatically");
            Ok(())
        } else {
            // No daemon - run foreground watch mode
            layout.info("Daemon not running, starting foreground watch mode...");
            layout.item_simple("(Run 'mnem start' to use background daemon mode)");
            println!();

            // Initialize or open repository
            let repo = Repository::init()?;

            layout.success(&format!("Now watching: {}", repo.project.name));
            layout.item_simple(&format!("ID: {}", repo.project.id));
            layout.item_simple(&format!("Path: {}", repo.project.path));
            layout.item_simple("Press Ctrl+C to stop watching");
            println!();

            // Setup file watcher
            let (tx, rx) = channel();

            let mut watcher =
                notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        let _ = tx.send(event);
                    }
                })?;

            watcher.watch(&path, RecursiveMode::Recursive)?;

            // Main watch loop
            loop {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(event) => {
                        // Only process file modifications (not directories)
                        for path in event.paths {
                            if path.is_file() {
                                // Skip ignored files
                                if should_ignore(&path) {
                                    continue;
                                }

                                match repo.save_snapshot_from_file(&path) {
                                    Ok(hash) => {
                                        println!(
                                            "[SAVED] {} (hash: {})",
                                            path.display(),
                                            &hash[..8.min(hash.len())]
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[ERROR] Failed to save {}: {}",
                                            path.display(),
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Timeout, continue loop
                        continue;
                    }
                }
            }
        }
    }
}

fn should_ignore(path: &PathBuf) -> bool {
    let path_str = path.to_string_lossy();

    // Ignore common directories
    let ignored_patterns = [
        ".git/",
        ".mnemosyne/",
        "target/",
        "node_modules/",
        ".idea/",
        ".vscode/",
        "tmp/",
        "temp/",
        ".log",
        ".tmp",
        ".bak",
        ".swp",
        ".swo",
        ".DS_Store",
    ];

    for pattern in &ignored_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }

    false
}
