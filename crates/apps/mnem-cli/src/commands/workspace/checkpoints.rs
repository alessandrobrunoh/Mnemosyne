use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct CheckpointsCommand;

impl Command for CheckpointsCommand {
    fn name(&self) -> &str {
        "checkpoints"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "List all manual checkpoints for the current project"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let layout = Layout::new();

        let base_dir = get_base_dir()?;
        let registry = ProjectRegistry::new(&base_dir)?;
        let projects = registry.list_projects();

        let cwd = env::current_dir()?;
        let p = projects
            .iter()
            .find(|p| {
                let p_path = Path::new(&p.path);
                cwd.starts_with(p_path) || p_path == cwd
            })
            .ok_or_else(|| anyhow::anyhow!("Not in a tracked project"))?;

        let repo = Repository::open(base_dir, p.path.clone().into())?;
        let checkpoints = repo.list_checkpoints().unwrap_or_default();

        layout.header("CHECKPOINTS");
        layout.section_start("ch", "Project Checkpoints");

        for (idx, cp) in checkpoints.iter().enumerate() {
            let id_tag = format!("{:02}", idx);
            let time_str = chrono::DateTime::parse_from_rfc3339(&cp.1)
                .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|_| cp.1.clone());

            let message =
                cp.2.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("No description");
            let content = format!("{}  {}", time_str, message);
            layout.row_list(&id_tag, &content);
        }

        layout.section_end();

        if checkpoints.is_empty() {
            layout.info("No checkpoints yet. Use 'mnem checkpoint' to create one.");
        }

        Ok(())
    }
}
