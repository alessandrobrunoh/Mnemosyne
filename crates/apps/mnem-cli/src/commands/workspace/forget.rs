use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry};

#[derive(Debug)]
pub struct ForgetCommand;

impl Command for ForgetCommand {
    fn name(&self) -> &str {
        "forget"
    }

    fn usage(&self) -> &str {
        "<id> [--prune]"
    }

    fn description(&self) -> &str {
        "Remove a project from the registry"
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
        let mut registry = ProjectRegistry::new(&base_dir)?;
        let projects = registry.list_projects();

        let p = projects
            .iter()
            .find(|p| p.id.starts_with(id_query) || p.id == *id_query)
            .ok_or_else(|| anyhow::anyhow!("Project not found with ID: {}", id_query))?;

        let prune = args.iter().any(|a| a == "--prune" || a == "-p");

        if prune {
            layout.warning(&format!(
                "This will DELETE all data for project: {}",
                p.path
            ));
            layout.item_simple("Type 'yes' to confirm...");

            let mut confirmation = String::new();
            std::io::stdin().read_line(&mut confirmation)?;

            if confirmation.trim().to_lowercase() != "yes" {
                layout.info("Operation cancelled.");
                return Ok(());
            }
        }

        registry.remove(&p.id)?;
        layout.success(&format!("Project '{}' removed from registry.", p.name));

        if prune {
            let repo_path = base_dir.join(&p.id);
            if repo_path.exists() {
                std::fs::remove_dir_all(&repo_path)?;
                layout.success("Project data deleted.");
            }
        }

        Ok(())
    }
}
