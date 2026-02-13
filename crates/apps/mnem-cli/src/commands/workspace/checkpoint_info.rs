use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct CheckpointInfoCommand;

impl Command for CheckpointInfoCommand {
    fn name(&self) -> &str {
        "checkpoint-info"
    }

    fn usage(&self) -> &str {
        "<id>"
    }

    fn description(&self) -> &str {
        "Show detailed information about a specific checkpoint"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();

        let cp_id_str = args
            .get(2)
            .ok_or_else(|| anyhow::anyhow!("Checkpoint ID is required"))?;

        let cp_id = cp_id_str.parse::<i64>()?;

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

        let cp = checkpoints
            .get(cp_id as usize)
            .ok_or_else(|| anyhow::anyhow!("Checkpoint not found"))?;

        layout.header("CHECKPOINT INFO");
        layout.section_start("ci", "Checkpoint Details");

        layout.row_property("Hash", &cp.0);
        let message =
            cp.2.as_ref()
                .map(|s| s.as_str())
                .unwrap_or("No description");
        layout.row_property("Message", message);

        let time_str = chrono::DateTime::parse_from_rfc3339(&cp.1)
            .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M:%S")))
            .unwrap_or_else(|_| cp.1.clone());
        layout.row_property("Created", &time_str);

        layout.section_end();
        layout.footer("Type 'mnem restore <checkpoint_id>' to restore this state.");
        Ok(())
    }
}
