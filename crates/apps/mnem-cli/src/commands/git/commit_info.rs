use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct CommitInfoCommand;

impl Command for CommitInfoCommand {
    fn name(&self) -> &str {
        "commit-info"
    }

    fn usage(&self) -> &str {
        "<hash>"
    }

    fn description(&self) -> &str {
        "Show detailed information about a specific Git commit"
    }

    fn group(&self) -> &str {
        "Git Integration"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();

        let commit_hash = args
            .get(2)
            .ok_or_else(|| anyhow::anyhow!("Commit hash is required"))?;

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
        let commits = repo.list_commits()?;
        let commits: Vec<_> = commits.into_iter().take(100).collect();

        let commit = commits
            .iter()
            .find(|c| c.0.starts_with(commit_hash))
            .ok_or_else(|| anyhow::anyhow!("Commit not found"))?;

        layout.header("COMMIT INFO");
        layout.section_start("gi", "Commit Details");

        layout.row_property("Hash", &commit.0);
        layout.row_property("Author", &commit.2);
        layout.row_property("Message", &commit.1);

        let time_str = chrono::DateTime::parse_from_rfc3339(&commit.3)
            .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M:%S")))
            .unwrap_or_else(|_| commit.3.clone());
        layout.row_property("Date", &time_str);

        layout.section_end();
        Ok(())
    }
}
