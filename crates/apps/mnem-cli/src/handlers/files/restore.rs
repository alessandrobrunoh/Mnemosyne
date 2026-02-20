use anyhow::Result;

use crate::handlers::files::history::compute_diff_stats;
use crate::ui::Layout;
use mnem_core::client::DaemonClient;
use mnem_core::protocol::SnapshotInfo;
use mnem_core::protocol::methods;
use mnem_core::storage::Repository;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Resolve the project root from an optional file argument.
/// Looks for a `.mnemosyne/tracked` file walking upward from the file's parent dir.
fn get_project_from_file(file: &Option<String>) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let start = if let Some(f) = file {
        let p = std::path::Path::new(f);
        let resolved = if p.is_relative() {
            cwd.join(p)
        } else {
            p.to_path_buf()
        };
        resolved.parent().map(|p| p.to_path_buf()).unwrap_or(cwd)
    } else {
        cwd
    };

    let tracked_file = start.join(".mnemosyne").join("tracked");
    if !tracked_file.exists() {
        return Err(anyhow::anyhow!(
            "Project not tracked: {:?}\n\nRun 'mnem track' to start tracking this project.",
            start
        ));
    }
    Ok(start)
}

fn cleanup_old_temp_files() {
    let temp_dir = std::env::temp_dir();
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(24 * 60 * 60))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        let name = entry.file_name();
                        if name.to_string_lossy().ends_with("_mnem.rs")
                            || name.to_string_lossy().ends_with("_mnem.tmp")
                        {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_r(
    file: Option<String>,
    version: Option<usize>,
    list: bool,
    undo: bool,
    to: Option<String>,
    symbol: Option<String>,
    checkpoint: Option<String>,
    branch: Option<String>,
    limit: Option<usize>,
) -> Result<()> {
    use mnem_core::config::ConfigManager;
    use mnem_core::env::get_base_dir;

    let layout = Layout::new();
    let base_dir = get_base_dir()?;
    let config = ConfigManager::new(&base_dir)?;
    let ide = config.config.ide;

    cleanup_old_temp_files();

    // Resolve project path (filesystem check only, no DB open)
    let project_path = match get_project_from_file(&file) {
        Ok(p) => p,
        Err(_) => {
            let cwd = std::env::current_dir()?;
            layout.header_dashboard("PROJECT NOT TRACKED");
            layout.section_branch("pr", "Project Folder");
            layout.row_labeled("◫", "Current Dir", &cwd.to_string_lossy());
            layout.section_end();
            layout.empty();
            layout.badge_error("ERROR", "This project is not tracked");
            layout.info_bright("Run 'mnem track' to start tracking this project.");
            return Ok(());
        }
    };

    // Try daemon first; fall back to direct DB only when daemon is not available.
    let daemon = DaemonClient::connect().ok();
    // If daemon connection fails with IO error (stale socket), try direct DB
    let repo_opt: Option<Repository> = if daemon.is_none() {
        match Repository::open(base_dir.clone(), project_path.clone()) {
            Ok(r) => Some(r),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("lock") || msg.contains("Database already open") {
                    layout.error("Cannot access history while daemon is running.");
                    layout.info_bright("Run 'mnem off' to stop the daemon first.");
                    return Ok(());
                }
                return Err(anyhow::anyhow!("{}", e));
            }
        }
    } else {
        None
    };

    // -----------------------------------------------------------------------
    // --checkpoint
    // -----------------------------------------------------------------------
    if let Some(ref cp) = checkpoint {
        if let Some(mut client) = daemon {
            let _ = client.call(
                methods::PROJECT_REVERT_V1,
                serde_json::json!({ "timestamp": cp }),
            )?;
            layout.success(&format!("Restored project from checkpoint {}", cp));
        } else if let Some(repo) = repo_opt.as_ref() {
            let count = repo.revert_to_checkpoint(cp)?;
            layout.success(&format!("Restored {} files from checkpoint {}", count, cp));
        }
        return Ok(());
    }

    // -----------------------------------------------------------------------
    // File operations
    // -----------------------------------------------------------------------
    let f = match file.as_ref() {
        Some(f) => f,
        None => {
            layout.error("Specify a file: mnem r <file> [version]");
            return Ok(());
        }
    };

    let clean_path = f.trim_start_matches(".\\").trim_start_matches("./");

    // --list
    if list {
        let full_path = if std::path::Path::new(f).is_absolute() {
            f.clone()
        } else {
            project_path.join(f).to_string_lossy().to_string()
        };

        // Try daemon (fast path — no temp files needed)
        if let Some(mut client) = daemon {
            match client.call(
                methods::SNAPSHOT_LIST,
                serde_json::json!({ "file_path": full_path }),
            ) {
                Ok(res) => {
                    match serde_json::from_value::<Vec<SnapshotInfo>>(res.clone()) {
                        Ok(mut history) => {
                            if let Some(ref br) = branch {
                                history.retain(|s| s.git_branch.as_deref().unwrap_or("main") == br);
                            }
                            let max = limit.unwrap_or(50);
                            history.truncate(max);

                            layout.header_dashboard("RESTORE VERSIONS");
                            layout.section_branch("fi", f);
                            layout.item_simple(&format!("Found {} versions", history.len()));

                            for (i, snap) in history.iter().enumerate() {
                                let hash_short =
                                    &snap.content_hash[..8.min(snap.content_hash.len())];
                                layout.row_version_with_link(
                                    i + 1,
                                    hash_short,
                                    &snap.content_hash,
                                    &snap.file_path,
                                    &snap.timestamp,
                                    i == 0,
                                    None,
                                    &ide,
                                );
                            }
                            layout.section_end();
                            layout.footer("Use 'mnem r <file> [version]' to restore");
                            return Ok(());
                        }
                        Err(parse_err) => {
                            // Debug: show what we got from daemon
                            layout.warning(&format!("Daemon parse error: {}", parse_err));
                            layout.info(&format!("Raw response: {}", res));
                        }
                    }
                }
                Err(e) => {
                    layout.warning(&format!("Daemon error: {e}"));
                }
            }
        }

        // Fallback: direct DB (daemon not running)
        let repo = match repo_opt {
            Some(r) => r,
            None => {
                layout.error("Cannot connect to daemon and cannot open local DB.");
                return Ok(());
            }
        };

        let mut history = match repo.get_history(clean_path) {
            Ok(h) => h,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("lock") || msg.contains("Database already open") {
                    layout.error("Cannot access history while daemon is running.");
                    layout.info_bright("Run 'mnem off' to stop the daemon first.");
                    return Ok(());
                }
                return Err(anyhow::anyhow!("{}", e));
            }
        };

        if let Some(ref br) = branch {
            history.retain(|s| s.git_branch.as_deref().unwrap_or("main") == br);
        }

        let max = limit.unwrap_or(50);
        history.truncate(max);

        layout.header_dashboard("RESTORE VERSIONS");
        layout.section_branch("fi", clean_path);

        if history.is_empty() {
            layout.warning("No versions found.");
            if let Some(ref br) = branch {
                layout.info(&format!("No versions on branch '{}'", br));
            }
            layout.section_end();
            return Ok(());
        }

        let extension = std::path::Path::new(clean_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Write temp files for IDE links (only in fallback / daemon-off path)
        let mut temp_files: BTreeMap<String, String> = BTreeMap::new();
        for snap in &history {
            if let Ok(content) = repo.get_content(&snap.content_hash) {
                let temp_filename = if !extension.is_empty() {
                    format!(
                        "{}_{}_mnem.{}",
                        std::path::Path::new(clean_path)
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file"),
                        &snap.content_hash[..8],
                        extension
                    )
                } else {
                    format!(
                        "{}_{}_mnem.tmp",
                        std::path::Path::new(clean_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file"),
                        &snap.content_hash[..8]
                    )
                };
                let temp_path = std::env::temp_dir().join(&temp_filename);

                let should_write = match fs::read(&temp_path) {
                    Ok(existing) => existing != content,
                    Err(_) => true,
                };

                if should_write && fs::write(&temp_path, &content).is_ok() {
                    temp_files.insert(
                        snap.content_hash.clone(),
                        temp_path.to_string_lossy().to_string(),
                    );
                } else if temp_path.exists() {
                    temp_files.insert(
                        snap.content_hash.clone(),
                        temp_path.to_string_lossy().to_string(),
                    );
                }
            }
        }

        let mut by_branch: BTreeMap<String, Vec<_>> = BTreeMap::new();
        for snap in &history {
            let br = snap
                .git_branch
                .clone()
                .unwrap_or_else(|| "main".to_string());
            by_branch.entry(br).or_default().push(snap);
        }

        for (branch_name, snaps) in &by_branch {
            let icon = if branch_name == "main" { "ma" } else { "br" };
            layout.section_branch(icon, branch_name);

            for (i, snap) in snaps.iter().enumerate() {
                let hash_short = &snap.content_hash[..8.min(snap.content_hash.len())];

                let ts_string = snap.timestamp.to_string();
                let date_time = if let Some(t_pos) = ts_string.find('T') {
                    let time_part = &ts_string[t_pos + 1..];
                    let time_trimmed = time_part.split('.').next().unwrap_or(time_part);
                    format!("{} {}", &ts_string[..t_pos], time_trimmed)
                } else {
                    ts_string.clone()
                };

                let file_to_open = temp_files
                    .get(&snap.content_hash)
                    .cloned()
                    .unwrap_or_else(|| project_path.join(clean_path).to_string_lossy().to_string());

                let prev_hash = snaps.get(i + 1).map(|s| s.content_hash.as_str());
                let diff_stats = compute_diff_stats(&repo, &snap.content_hash, prev_hash);

                layout.row_version_with_link(
                    i + 1,
                    hash_short,
                    &snap.content_hash,
                    &file_to_open,
                    &date_time,
                    i == 0,
                    diff_stats,
                    &ide,
                );
            }
            layout.section_end();
        }

        layout.footer("Click on hash to open that version in IDE");
        return Ok(());
    }

    // -----------------------------------------------------------------------
    // Restore helpers (daemon-first)
    // -----------------------------------------------------------------------
    let do_restore = |daemon_opt: Option<DaemonClient>,
                      repo_ref: Option<&Repository>,
                      hash: &str,
                      sym: Option<&String>|
     -> Result<()> {
        let target = project_path.join(clean_path).to_string_lossy().to_string();
        if let Some(mut c) = daemon_opt {
            if let Some(s) = sym {
                let _ = c.call(
                    methods::SNAPSHOT_RESTORE_SYMBOL_V1,
                    serde_json::json!({ "content_hash": hash, "target_path": target, "symbol_name": s }),
                )?;
            } else {
                let _ = c.call(
                    methods::SNAPSHOT_RESTORE_V1,
                    serde_json::json!({ "content_hash": hash, "target_path": target }),
                )?;
            }
        } else if let Some(repo) = repo_ref {
            if let Some(s) = sym {
                repo.restore_symbol(clean_path, hash, s)?;
            } else {
                repo.restore_file(hash, clean_path)?;
            }
        } else {
            anyhow::bail!("Neither daemon nor local DB is available");
        }
        Ok(())
    };

    // --undo
    if undo {
        let history = get_history_for_restore(
            daemon.as_ref().map(|_| ()),
            repo_opt.as_ref(),
            &project_path,
            clean_path,
            &mut DaemonClient::connect().ok(),
        )?;
        if history.len() < 2 {
            anyhow::bail!("No previous version to restore");
        }
        let prev_hash = history[1].content_hash.clone();
        let prev_ts = history[1].timestamp.clone();
        do_restore(daemon, repo_opt.as_ref(), &prev_hash, None)?;
        layout.success(&format!(
            "Restored {} to version from {}",
            clean_path, prev_ts
        ));
        return Ok(());
    }

    // --to <hash>
    if let Some(ref hash) = to {
        do_restore(daemon, repo_opt.as_ref(), hash, symbol.as_ref())?;
        if let Some(ref sym) = symbol {
            layout.success(&format!(
                "Restored symbol '{}' in {} to {}",
                sym,
                clean_path,
                &hash[..8.min(hash.len())]
            ));
        } else {
            layout.success(&format!(
                "Restored {} to {}",
                clean_path,
                &hash[..8.min(hash.len())]
            ));
        }
        return Ok(());
    }

    // <version>
    if let Some(v) = version {
        let history = get_history_for_restore(
            daemon.as_ref().map(|_| ()),
            repo_opt.as_ref(),
            &project_path,
            clean_path,
            &mut DaemonClient::connect().ok(),
        )?;
        if v == 0 || v > history.len() {
            anyhow::bail!("Invalid version number. Use --list to see available versions.");
        }
        let target_hash = history[v - 1].content_hash.clone();
        do_restore(daemon, repo_opt.as_ref(), &target_hash, symbol.as_ref())?;
        if let Some(ref sym) = symbol {
            layout.success(&format!(
                "Restored symbol '{}' in {} to version {}",
                sym, clean_path, v
            ));
        } else {
            layout.success(&format!("Restored {} to version {}", clean_path, v));
        }
        return Ok(());
    }

    // No action specified
    layout.usage("r", "<file> [version]");
    layout.info("Examples:");
    layout.item_simple("mnem r main.rs --list");
    layout.item_simple("mnem r main.rs --list --branch test");
    layout.item_simple("mnem r main.rs --list --limit 10");
    layout.item_simple("mnem r main.rs 3");
    layout.item_simple("mnem r main.rs --undo");
    layout.item_simple("mnem r main.rs --to <hash>");

    Ok(())
}

/// Get snapshot history, preferring daemon then falling back to direct DB.
fn get_history_for_restore(
    daemon_present: Option<()>,
    repo_opt: Option<&Repository>,
    project_path: &PathBuf,
    clean_path: &str,
    client: &mut Option<DaemonClient>,
) -> Result<Vec<SnapshotInfo>> {
    if daemon_present.is_some() {
        if let Some(c) = client.as_mut() {
            let full_path = project_path.join(clean_path).to_string_lossy().to_string();
            let res = c.call(
                methods::SNAPSHOT_LIST,
                serde_json::json!({ "file_path": full_path }),
            )?;
            match serde_json::from_value::<Vec<SnapshotInfo>>(res.clone()) {
                Ok(history) => return Ok(history),
                Err(e) => {
                    eprintln!("Daemon parse error in get_history_for_restore: {}", e);
                    eprintln!("Raw response: {:?}", res);
                }
            }
        }
    }
    if let Some(repo) = repo_opt {
        let snaps = repo.get_history(clean_path)?;
        return Ok(snaps
            .into_iter()
            .map(|s| SnapshotInfo {
                id: s.id,
                file_path: s.file_path,
                timestamp: s.timestamp,
                content_hash: s.content_hash,
                git_branch: s.git_branch,
                commit_hash: s.commit_hash,
                commit_message: s.commit_message,
            })
            .collect());
    }
    anyhow::bail!("Neither daemon nor local DB is available to fetch history")
}
