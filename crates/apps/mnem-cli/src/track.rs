use anyhow::Result;

use crate::ui::Layout;

pub fn handle_track(list: bool, _remove: bool, _id: Option<String>) -> Result<()> {
    use mnem_core::client::daemon_running;
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::registry::ProjectRegistry;

    let layout = Layout::new();
    let base_dir = get_base_dir()?;
    let mut registry = ProjectRegistry::new(&base_dir)?;

    if list {
        let projects = registry.list_projects();
        layout.header_dashboard("TRACKED PROJECTS");

        if projects.is_empty() {
            layout.warning("No tracked projects.");
            layout.empty();
            layout.info("Use 'mnem track' in a project directory to start tracking");
        } else {
            for p in &projects {
                layout.bullet_purple(&p.name);
                layout.row_file_path(&p.path);
            }
        }
        layout.section_end();
        return Ok(());
    }

    let cwd = std::env::current_dir()?;
    let tracked_file = cwd.join(".mnemosyne").join("tracked");

    if tracked_file.exists() {
        layout.header_dashboard("ALREADY TRACKED");
        layout.info_bright("This project is already tracked");
        layout.empty();

        if let Ok(content) = std::fs::read_to_string(&tracked_file) {
            let mut name = String::new();
            let mut id = String::new();
            let mut path = String::new();

            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("project_name:") {
                    name = line
                        .split(':')
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join(":")
                        .trim()
                        .to_string();
                } else if line.starts_with("project_id:") {
                    id = line
                        .split(':')
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join(":")
                        .trim()
                        .to_string();
                } else if line.starts_with("path:") && !line.starts_with("project_") {
                    path = line
                        .split(':')
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join(":")
                        .trim()
                        .to_string();
                }
            }

            if !id.is_empty() {
                layout.row_labeled("◆", "ID", &id);
            }
            if !name.is_empty() {
                layout.row_labeled("◫", "Name", &name);
            }
            if !path.is_empty() {
                layout.row_labeled("◫", "Path", &path);
            }
        }
        layout.empty();
        layout.badge_info("INFO", "This project is already being tracked");
        return Ok(());
    }

    if !daemon_running() {
        layout.info("Starting daemon first...");
        mnem_core::client::ensure_daemon()?;
    }

    let project = registry.get_or_create(&cwd)?;

    layout.header_dashboard("PROJECT TRACKED");
    layout.success_bright(&format!("✓ Now tracking: {}", project.name));
    layout.empty();
    layout.row_labeled("◫", "Path", &project.path);
    layout.row_labeled("◆", "ID", &project.id);
    layout.empty();
    layout.badge_success("OK", "Tracking started");

    Ok(())
}
