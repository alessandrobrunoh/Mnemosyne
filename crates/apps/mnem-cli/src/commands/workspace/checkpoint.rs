use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct CheckpointCommand;

impl Command for CheckpointCommand {
    fn name(&self) -> &str {
        "checkpoint"
    }

    fn usage(&self) -> &str {
        "[<message>]"
    }

    fn description(&self) -> &str {
        "Create a manual project-wide semantic snapshot"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
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

        let message = args
            .get(2)
            .map(|s| s.as_str())
            .unwrap_or("Manual checkpoint");

        repo.create_checkpoint(Some(message))?;
        layout.success("Checkpoint created successfully!");
        layout.item_simple(&format!("Message: {}", message));

        Ok(())
    }
}
