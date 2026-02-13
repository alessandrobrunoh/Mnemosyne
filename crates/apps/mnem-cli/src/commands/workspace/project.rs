use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::path::Path;

#[derive(Debug)]
pub struct ProjectCommand;

impl Command for ProjectCommand {
    fn name(&self) -> &str {
        "project"
    }

    fn usage(&self) -> &str {
        "<id>"
    }

    fn description(&self) -> &str {
        "Show detailed information about a specific project"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();

        let id_query = args
            .get(2)
            .ok_or_else(|| anyhow::anyhow!("Project ID is required"))?;

        let base_dir = get_base_dir()?;
        let registry = ProjectRegistry::new(&base_dir)?;
        let projects = registry.list_projects();

        let p = projects
            .iter()
            .find(|p| p.id.starts_with(id_query) || p.id == *id_query)
            .ok_or_else(|| anyhow::anyhow!("Project not found with ID: {}", id_query))?;

        let repo = Repository::open(base_dir, p.path.clone().into())?;
        let size = repo.get_project_size()?;
        let files = repo.list_files(None, None)?;
        let branches = repo.list_branches()?;
        let checkpoints = repo.list_checkpoints().unwrap_or_default();

        layout.header("PROJECT INTELLIGENCE");
        layout.section_start("inf", "General Information");

        let name = Path::new(&p.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let info_stats = [
            ("Name", name),
            ("ID", &p.id),
            ("Root", &p.path),
            ("Files", &files.len().to_string()),
            ("Branches", &branches.len().to_string()),
            ("Checkpoints", &checkpoints.len().to_string()),
            ("Size", &format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)),
        ];

        for (key, val) in info_stats {
            layout.row_property(key, val);
        }

        layout.section_end();

        let last_open = chrono::DateTime::parse_from_rfc3339(&p.last_open)
            .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
            .unwrap_or_else(|_| p.last_open.clone());
        layout.item_simple(&format!("Last Open: {}", last_open));

        layout.footer("Type 'mnem <id> history' to see project activity.");
        Ok(())
    }
}
