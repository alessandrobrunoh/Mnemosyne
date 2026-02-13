use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct StatisticsCommand;

impl Command for StatisticsCommand {
    fn name(&self) -> &str {
        "statistics"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Show cool and interesting project metrics"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["stats"]
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

        // Get statistics
        let total_size = repo.get_project_size()?;
        let commits = repo.list_commits()?;
        let checkpoints = repo.list_checkpoints().unwrap_or_default();
        let branches = repo.list_branches()?;
        let files = repo.list_files(None, None)?;
        let recent_activity = repo.get_recent_activity(100)?;

        layout.header("PROJECT STATISTICS");
        layout.section_start("st", "Statistics Overview");

        let stats = [
            ("Snapshots", recent_activity.len().to_string()),
            ("Files", files.len().to_string()),
            ("Branches", branches.len().to_string()),
            ("Commits", commits.len().to_string()),
            ("Checkpoints", checkpoints.len().to_string()),
            (
                "Vault Size",
                format!("{:.2} MB", total_size as f64 / 1024.0 / 1024.0),
            ),
        ];

        for (key, val) in stats {
            layout.row_property(key, &val);
        }

        layout.section_end();

        if !recent_activity.is_empty() {
            layout.section_start("ac", "Recent Activity");
            for snapshot in recent_activity.iter().take(10) {
                let time_str = chrono::DateTime::parse_from_rfc3339(&snapshot.timestamp)
                    .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
                    .unwrap_or_else(|_| snapshot.timestamp.clone());
                layout.row_list(&time_str, &snapshot.file_path);
            }
            layout.section_end();
        }

        Ok(())
    }
}
