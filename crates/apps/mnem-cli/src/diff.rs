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

pub fn handle_d(file: Option<String>, from: Option<usize>, to: Option<usize>) -> Result<()> {
    use mnem_core::storage::Repository;

    let layout = Layout::new();
    let (base_dir, cwd) = match check_project_tracked(&layout) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let repo = Repository::open(base_dir, cwd)?;

    if let Some(ref f) = file {
        let clean_path = if f.starts_with(".\\") {
            &f[2..]
        } else if f.starts_with("./") {
            &f[2..]
        } else {
            f
        };
        let history = repo.get_history(clean_path)?;

        if history.is_empty() {
            layout.header_dashboard("DIFF");
            layout.warning(&format!("No history found for {}", clean_path));
            layout.empty();
            layout.badge_info("TIP", "Edit the file to create history");
            return Ok(());
        }

        let from_ver = from.unwrap_or(2).min(history.len());
        let to_ver = to.unwrap_or(1).min(history.len());

        if from_ver < to_ver {
            anyhow::bail!("'from' version must be newer than 'to' version");
        }

        let from_hash = &history[from_ver - 1].content_hash;
        let to_hash = &history[to_ver - 1].content_hash;

        layout.header_dashboard("DIFF");
        layout.section_timeline("df", clean_path);
        layout.badge_info(
            &format!("v{} → v{}", from_ver, to_ver),
            &format!("{} → {}", &from_hash[..8], &to_hash[..8]),
        );

        match repo.get_file_diff(clean_path, Some(from_hash), to_hash) {
            Ok(diff) => {
                if diff.is_empty() {
                    layout.info(&format!(
                        "No differences between versions {} and {}",
                        from_ver, to_ver
                    ));
                    layout.empty();
                    layout.badge_success("SAME", "Files are identical");
                } else {
                    layout.empty();
                    for line in diff.lines() {
                        if line.starts_with('+') && !line.starts_with("+++") {
                            layout.row_diff_add(line);
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            layout.row_diff_remove(line);
                        } else if line.starts_with("@@") {
                            layout.row_diff_header(line);
                        } else if line.starts_with("+++") || line.starts_with("---") {
                            layout.row_diff_context(line);
                        } else {
                            layout.item_simple(line);
                        }
                    }
                }
            }
            Err(e) => {
                layout.error_bright(&format!("Unable to generate diff: {}", e));
            }
        }
        layout.section_end();
    } else {
        layout.usage("d", "<file> [--from N] [--to N]");
        layout.empty();
        layout.badge_info("EXAMPLE", "mnem d main.rs --from 3 --to 1");
    }

    Ok(())
}
