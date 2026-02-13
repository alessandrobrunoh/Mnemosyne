use crate::commands::Command;
use crate::ui::Layout;
use crate::utils;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct HistoryCommand;

impl Command for HistoryCommand {
    fn name(&self) -> &str {
        "history"
    }

    fn usage(&self) -> &str {
        "[--limit <n>]"
    }

    fn description(&self) -> &str {
        "Show project history (snapshots over time)"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        self.execute_with_override(args, None)
    }
}

impl HistoryCommand {
    pub fn execute_with_override(
        &self,
        args: &[String],
        project_path_override: Option<String>,
    ) -> Result<()> {
        let layout = Layout::new();
        let common_args = utils::parse_common_args(args);

        let base_dir = get_base_dir()?;
        let registry = ProjectRegistry::new(&base_dir)?;
        let projects = registry.list_projects();

        let project_path = if let Some(override_path) = project_path_override {
            override_path
        } else {
            let cwd = env::current_dir()?;
            let p = projects
                .iter()
                .find(|p| {
                    let p_path = Path::new(&p.path);
                    cwd.starts_with(p_path) || p_path == cwd
                })
                .ok_or_else(|| anyhow::anyhow!("Not in a tracked project"))?;
            p.path.clone()
        };

        let repo = Repository::open(base_dir, project_path.into())?;
        let snapshots = repo.get_recent_activity(common_args.limit)?;

        layout.header("PROJECT HISTORY");
        layout.section_start("hi", "Snapshots");

        for snapshot in snapshots {
            let time_str = chrono::DateTime::parse_from_rfc3339(&snapshot.timestamp)
                .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M:%S")))
                .unwrap_or_else(|_| snapshot.timestamp.clone());

            layout.row_snapshot(
                &snapshot.content_hash[..8.min(snapshot.content_hash.len())],
                &time_str,
            );
        }

        layout.section_end();
        layout.footer("Type 'mnem timeline <symbol>' to see semantic evolution.");
        Ok(())
    }
}
