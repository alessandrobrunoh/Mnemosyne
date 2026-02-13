
use anyhow::Result;

use crate::handlers::files::history::compute_diff_stats;
use crate::ui::Layout;
use mnem_core::storage::Repository;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

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
    let (_, cwd, repo) = match check_project_tracked(&layout) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let base_dir = get_base_dir()?;
    let config = ConfigManager::new(&base_dir)?;
    let ide = config.config.ide;

    cleanup_old_temp_files();

    if list {
        if let Some(ref f) = file {
            let clean_path = if f.starts_with(".\\") {
                &f[2..]
            } else if f.starts_with("./") {
                &f[2..]
            } else {
                f
            };
            let mut history = repo.get_history(clean_path)?;

            if let Some(ref br) = branch {
                history.retain(|s| s.git_branch.as_deref().unwrap_or("main") == br);
            }

            let limit = limit.unwrap_or(50);
            history.truncate(limit);

            layout.header_dashboard("RESTORE VERSIONS");
            layout.section_branch("fi", clean_path);

            if history.is_empty() {
                layout.warning("No versions found.");
                if branch.is_some() {
                    layout.info(&format!("No versions on branch '{}'", branch.unwrap()));
                }
                layout.section_end();
                return Ok(());
            }

            let extension = std::path::Path::new(clean_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

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

                    let should_write = if let Ok(existing) = fs::read(&temp_path) {
                        existing != content
                    } else {
                        true
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
                let branch_name = snap
                    .git_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string());
                by_branch.entry(branch_name).or_default().push(snap);
            }

            for (branch_name, snaps) in &by_branch {
                let branch_icon = if branch_name == "main" { "ma" } else { "br" };
                layout.section_branch(branch_icon, branch_name);

                for (i, snap) in snaps.iter().enumerate() {
                    let hash_short = &snap.content_hash[..8.min(snap.content_hash.len())];

                    let ts_string = snap.timestamp.to_string();
                    let timestamp_parts: Vec<&str> = ts_string.split('T').collect();
                    let date_time = if timestamp_parts.len() > 1 {
                        let time_parts: Vec<&str> = timestamp_parts[1].split('.').collect();
                        format!("{} {}", timestamp_parts[0], time_parts[0])
                    } else {
                        ts_string.clone()
                    };

                    let file_to_open = temp_files
                        .get(&snap.content_hash)
                        .cloned()
                        .unwrap_or_else(|| cwd.join(clean_path).to_string_lossy().to_string());

                    let prev_snap = snaps.get(i + 1);
                    let prev_hash = prev_snap.map(|s| s.content_hash.as_str());
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
        } else {
            layout.error("Specify a file with: mnem r <file> --list");
        }
        return Ok(());
    }

    if let Some(ref cp) = checkpoint {
        let count = repo.revert_to_checkpoint(cp)?;
        layout.success(&format!("Restored {} files from checkpoint {}", count, cp));
        return Ok(());
    }

    if let Some(ref f) = file {
        let clean_path = if f.starts_with(".\\") {
            &f[2..]
        } else if f.starts_with("./") {
            &f[2..]
        } else {
            f
        };

        if undo {
            let history = repo.get_history(clean_path)?;
            if history.len() < 2 {
                anyhow::bail!("No previous version to restore");
            }
            let prev = &history[1];
            repo.restore_file(&prev.content_hash, clean_path)?;
            layout.success(&format!(
                "Restored {} to version from {}",
                clean_path, prev.timestamp
            ));
            return Ok(());
        }

        if let Some(v) = version {
            let history = repo.get_history(clean_path)?;
            if v == 0 || v > history.len() {
                anyhow::bail!("Invalid version number. Use --list to see available versions.");
            }
            let target = &history[v - 1];

            if let Some(ref sym) = symbol {
                repo.restore_symbol(clean_path, &target.content_hash, sym)?;
                layout.success(&format!(
                    "Restored symbol '{}' in {} to version {}",
                    sym, clean_path, v
                ));
            } else {
                repo.restore_file(&target.content_hash, clean_path)?;
                layout.success(&format!("Restored {} to version {}", clean_path, v));
            }
            return Ok(());
        }

        if let Some(ref hash) = to {
            if let Some(ref sym) = symbol {
                repo.restore_symbol(clean_path, hash, sym)?;
                layout.success(&format!(
                    "Restored symbol '{}' in {} to {}",
                    sym,
                    clean_path,
                    &hash[..8.min(hash.len())]
                ));
            } else {
                repo.restore_file(hash, clean_path)?;
                layout.success(&format!(
                    "Restored {} to {}",
                    clean_path,
                    &hash[..8.min(hash.len())]
                ));
            }
            return Ok(());
        }

        layout.usage("r", "<file> [version]");
        layout.info("Examples:");
        layout.item_simple("mnem r main.rs --list");
        layout.item_simple("mnem r main.rs --list --branch test");
        layout.item_simple("mnem r main.rs --list --limit 10");
        layout.item_simple("mnem r main.rs 3");
        layout.item_simple("mnem r main.rs --undo");
    } else {
        layout.error("Specify a file: mnem r <file> [version]");
    }

    Ok(())
}
