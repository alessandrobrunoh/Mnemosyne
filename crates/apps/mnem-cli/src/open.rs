use anyhow::Result;

use crate::ui::Layout;
use std::path::PathBuf;

fn check_project_tracked(layout: &Layout) -> Result<Option<(PathBuf, PathBuf)>> {
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
        return Ok(None);
    }

    Ok(Some((base_dir, cwd)))
}

pub fn handle_open(
    file: Option<String>,
    at: Option<usize>,
    checkpoint: Option<String>,
) -> Result<()> {
    use mnem_core::config::ConfigManager;
    use mnem_core::storage::Repository;

    let layout = Layout::new();
    let tracked = match check_project_tracked(&layout)? {
        Some(r) => r,
        None => return Ok(()),
    };
    let (base_dir, cwd) = tracked;
    let config = ConfigManager::new(&base_dir)?;
    let ide = config.config.ide;

    let repo = Repository::open(base_dir.clone(), cwd.clone()).ok();

    if let Some(ref cp) = checkpoint {
        layout.header_dashboard("OPEN CHECKPOINT");
        layout.info(&format!("Opening checkpoint {} in {}...", cp, ide.as_str()));
        return Ok(());
    }

    if let Some(ref f) = file {
        if f.len() >= 6 && f.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Some(repo) = repo {
                if let Ok(content) = repo.get_content(f) {
                    let filename = format!("snapshot_{}.rs", &f[..8]);
                    let temp_path = std::env::temp_dir().join(filename);
                    std::fs::write(&temp_path, content)?;

                    open_file(&ide, &temp_path.to_string_lossy(), at);
                    layout.success_bright(&format!(
                        "✓ Opened snapshot {} in {}",
                        &f[..8],
                        ide.as_str()
                    ));
                    return Ok(());
                }
            }
        }

        let file_path = if std::path::Path::new(f).is_absolute() {
            f.clone()
        } else {
            cwd.join(f).to_string_lossy().to_string()
        };

        open_file(&ide, &file_path, at);
        layout.success_bright(&format!("✓ Opened in {}", ide.as_str()));
    } else {
        layout.info(&format!("Opening {} in {}...", cwd.display(), ide.as_str()));
        open_project(&ide, &cwd);
        layout.success_bright(&format!("✓ Opened project in {}", ide.as_str()));
    }

    Ok(())
}

fn open_file(ide: &mnem_core::config::Ide, path: &str, line: Option<usize>) {
    let cmd = if let Some(v) = line {
        format!("{} +{} {}", ide.command_name(), v, path)
    } else {
        format!("{} {}", ide.command_name(), path)
    };

    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &cmd])
            .spawn()
            .ok();
    }

    #[cfg(not(windows))]
    {
        std::process::Command::new(ide.command_name())
            .arg(path)
            .spawn()
            .ok();
    }
}

fn open_project(ide: &mnem_core::config::Ide, path: &std::path::Path) {
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &ide.command_name()])
            .current_dir(path)
            .spawn()
            .ok();
    }

    #[cfg(not(windows))]
    {
        std::process::Command::new(ide.command_name())
            .current_dir(path)
            .spawn()
            .ok();
    }
}
