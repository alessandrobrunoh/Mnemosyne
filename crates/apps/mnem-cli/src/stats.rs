use anyhow::Result;

use crate::ui::Layout;

pub fn handle_stats(_all: bool) -> Result<()> {
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::Repository;

    let layout = Layout::new();
    let base_dir = get_base_dir()?;
    let cwd = std::env::current_dir()?;
    let repo = Repository::open(base_dir, cwd)?;

    let history = repo.get_recent_activity(1000)?;
    let files: std::collections::HashSet<_> = history.iter().map(|s| &s.file_path).collect();

    layout.header_dashboard("STATISTICS");

    layout.section_timeline("st", "Overview");
    layout.row_metric("◫", "Snapshots", &history.len().to_string());
    layout.row_metric("◫", "Files", &files.len().to_string());
    layout.row_metric(
        "◫",
        "Size",
        &format!(
            "{:.2} MB",
            repo.get_project_size()? as f64 / 1024.0 / 1024.0
        ),
    );
    layout.section_end();

    layout.empty();
    layout.badge_info("TIP", "Use 'mnem h' to view file history");

    Ok(())
}
