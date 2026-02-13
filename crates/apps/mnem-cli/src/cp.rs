use anyhow::Result;

use crate::ui::Layout;
use std::path::PathBuf;

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

pub fn handle_cp(
    message: Option<String>,
    list: bool,
    id: Option<String>,
    info: bool,
    remove: bool,
    restore: bool,
) -> Result<()> {
    use mnem_core::storage::Repository;

    let layout = Layout::new();
    let (base_dir, cwd) = match check_project_tracked(&layout) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let repo = Repository::open(base_dir.clone(), cwd)?;

    if list {
        let cps = repo.list_checkpoints()?;
        layout.header_dashboard("CHECKPOINTS");
        layout.section_timeline("cp", "Project Checkpoints");

        if cps.is_empty() {
            layout.warning("No checkpoints found.");
            layout.empty();
            layout.info("Use 'mnem cp <message>' to create a checkpoint");
        } else {
            for (hash, timestamp, msg) in cps {
                let hash_short = &hash[..8.min(hash.len())];
                let timestamp_parts: Vec<&str> = timestamp.split('T').collect();
                let date_time = if timestamp_parts.len() > 1 {
                    let time_parts: Vec<&str> = timestamp_parts[1].split('.').collect();
                    format!("{} {}", timestamp_parts[0], time_parts[0])
                } else {
                    timestamp
                };

                layout.row_history(
                    hash_short,
                    &date_time,
                    msg.as_deref().unwrap_or("No description"),
                    false,
                );
            }
        }
        layout.section_end();
        return Ok(());
    }

    let target_id = if (info || remove || restore) && id.is_none() {
        message.as_ref()
    } else {
        id.as_ref()
    };

    if let Some(checkpoint_id) = target_id {
        if info {
            if let Some((ts, states_json, desc)) = repo.get_checkpoint_details(checkpoint_id)? {
                let files: Vec<(String, String)> = serde_json::from_str(&states_json)?;

                layout.header_dashboard("CHECKPOINT INFO");
                layout.section_timeline("cp", checkpoint_id);

                let timestamp_parts: Vec<&str> = ts.split('T').collect();
                let date_time = if timestamp_parts.len() > 1 {
                    let time_parts: Vec<&str> = timestamp_parts[1].split('.').collect();
                    format!("{} {}", timestamp_parts[0], time_parts[0])
                } else {
                    ts.to_string()
                };

                layout.row_labeled("◷", "Created", &date_time);
                if let Some(d) = desc {
                    layout.row_labeled("◆", "Description", &d);
                }
                layout.row_metric("◫", "Files Count", &files.len().to_string());

                layout.section_timeline("fi", "Files in Checkpoint");
                for (path, _) in files.iter().take(10) {
                    let p = path.replace("\\\\?\\", "");
                    let p_path = std::path::Path::new(&p);
                    let display_path = if let Ok(rel) = p_path.strip_prefix(&base_dir) {
                        rel.to_string_lossy().to_string()
                    } else {
                        p
                    };
                    layout.bullet_cyan(&display_path);
                }
                if files.len() > 10 {
                    layout.item_yellow(&format!("... and {} more files", files.len() - 10));
                }
                layout.section_end();
                layout.empty();
                layout.badge_info(
                    "INFO",
                    "Use 'mnem cp <id> --restore' to revert to this checkpoint",
                );
            } else {
                layout.error_bright("Checkpoint not found");
            }
            return Ok(());
        }

        if remove {
            let deleted = repo.delete_checkpoint(checkpoint_id)?;
            if deleted {
                layout.success_bright(&format!(
                    "✓ Removed checkpoint {}",
                    &checkpoint_id[..8.min(checkpoint_id.len())]
                ));
            } else {
                layout.error_bright("Checkpoint not found");
            }
            return Ok(());
        }

        if restore {
            let count = repo.revert_to_checkpoint(checkpoint_id)?;
            layout.success_bright(&format!(
                "✓ Restored {} files from checkpoint {}",
                count,
                &checkpoint_id[..8.min(checkpoint_id.len())]
            ));
            layout.empty();
            layout.badge_success("DONE", "Files restored successfully");
            return Ok(());
        }
    }

    if info || remove || restore {
        layout.error("Specify a checkpoint ID for this operation.");
        layout.usage("cp", "<id> --info|--remove|--restore");
        return Ok(());
    }

    let hash = repo.create_checkpoint(message.as_deref())?;
    layout.header_dashboard("CHECKPOINT CREATED");
    layout.success_bright(&format!("✓ Created checkpoint {}", &hash[..8]));
    layout.empty();
    layout.badge_success("OK", "Checkpoint saved successfully");

    Ok(())
}
