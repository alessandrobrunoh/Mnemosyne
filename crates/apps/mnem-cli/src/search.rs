use anyhow::Result;

use crate::ui::Layout;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn check_project_tracked(layout: &Layout) -> Result<(PathBuf, PathBuf)> {
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

    Ok((base_dir, cwd))
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
                        if name.to_string_lossy().ends_with("_mnem.tmp") {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_s(
    query: Option<String>,
    _file: Option<String>,
    limit: Option<usize>,
    semantic: bool,
) -> Result<()> {
    use mnem_core::config::ConfigManager;
    use mnem_core::storage::Repository;
    use std::collections::BTreeMap;

    let layout = Layout::new();
    let (base_dir, cwd) = match check_project_tracked(&layout) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let config = ConfigManager::new(&base_dir)?;
    let ide = config.config.ide;
    let repo = Repository::open(base_dir, cwd)?;

    // Clean up old temp files
    cleanup_old_temp_files();

    let query = query.ok_or_else(|| anyhow::anyhow!("Specify a search query"))?;

    if semantic {
        let results = repo.find_symbols(&query)?;
        layout.header_dashboard(&format!("SEMANTIC: {}", query));

        if results.is_empty() {
            layout.warning("No results found.");
        } else {
            for r in results.into_iter().take(limit.unwrap_or(20)) {
                layout.row_list(&format!("{}:{}", r.file_path, r.start_line), &r.name);
            }
        }
    } else {
        let results = repo.grep_contents(&query, None)?;
        layout.header_dashboard(&format!("SEARCH: {}", query));

        if results.is_empty() {
            layout.warning("No results found.");
            layout.empty();
            layout.badge_info("TIP", "Try a different search term");
        } else {
            let mut grouped: BTreeMap<String, Vec<&_>> = BTreeMap::new();
            for r in &results {
                grouped.entry(r.file_path.clone()).or_default().push(r);
            }

            let limit_val = limit.unwrap_or(10);
            let mut shown_files = 0;

            for (path, matches) in grouped {
                if shown_files >= limit_val {
                    break;
                }
                shown_files += 1;

                let filename = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&path);

                layout.section_timeline("fi", filename);
                layout.row_file_path(&path);

                // Get snapshot info and pre-create temp file
                if let Ok(hist) = repo.get_history(&path) {
                    if let Some(snap) = hist.first() {
                        let hash_short = &snap.content_hash[..8.min(snap.content_hash.len())];
                        let timestamp = snap.timestamp.to_string();
                        let parts: Vec<&str> = timestamp.split('T').collect();
                        let date_time = if parts.len() > 1 {
                            let time_parts: Vec<&str> = parts[1].split('.').collect();
                            format!("{} {}", parts[0], time_parts[0])
                        } else {
                            timestamp
                        };

                        let branch = snap.git_branch.as_deref().unwrap_or("main");

                        layout.row_history(
                            hash_short,
                            &date_time,
                            &format!("{}   ({} matches)", branch, matches.len()),
                            true,
                        );

                        // Create temp file for snapshot version (reuse if exists)
                        let temp_path = if let Ok(content) = repo.get_content(&snap.content_hash) {
                            // Get original file extension
                            let extension = std::path::Path::new(&path)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("");

                            let temp_filename = if !extension.is_empty() {
                                format!(
                                    "{}_{}_mnem.{}",
                                    std::path::Path::new(&path)
                                        .file_stem()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("file"),
                                    &snap.content_hash[..8],
                                    extension
                                )
                            } else {
                                format!(
                                    "{}_{}_mnem.tmp",
                                    std::path::Path::new(&path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("file"),
                                    &snap.content_hash[..8]
                                )
                            };
                            let tp = std::env::temp_dir().join(&temp_filename);

                            // Only write if doesn't exist or content differs
                            let content_bytes = content.clone();
                            let should_write = if let Ok(existing) = std::fs::read(&tp) {
                                existing != content_bytes
                            } else {
                                true
                            };

                            if should_write {
                                let _ = std::fs::write(&tp, &content);
                            }
                            tp.to_string_lossy().to_string()
                        } else {
                            path.clone()
                        };

                        layout.row_hash_link(hash_short, &snap.content_hash, &temp_path, &ide);
                    }
                }

                for m in matches.iter().take(5) {
                    layout.row_search_match(m.line_number, &m.content);
                }
                if matches.len() > 5 {
                    layout.item_yellow(&format!("... and {} more matches", matches.len() - 5));
                }
                layout.section_end();
            }

            layout.empty();
            layout.badge_info("HINT", "Click on hash to open that version in IDE");
        }
    }

    Ok(())
}
