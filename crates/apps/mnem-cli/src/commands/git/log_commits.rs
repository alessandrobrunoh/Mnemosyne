use crate::commands::Command;
use crate::ui::Layout;
use crate::utils;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct LogCommitsCommand;

impl Command for LogCommitsCommand {
    fn name(&self) -> &str {
        "log-commits"
    }

    fn usage(&self) -> &str {
        "[--limit <n>]"
    }

    fn description(&self) -> &str {
        "Show commits with their modified files"
    }

    fn group(&self) -> &str {
        "Git Integration"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        let common_args = utils::parse_common_args(args);

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
        let commits: Vec<_> = commits.into_iter().take(common_args.limit).collect();

        layout.header("COMMIT LOG");
        layout.section_start("gl", "Commits with Files");

        for commit in &commits {
            let time_str = chrono::DateTime::parse_from_rfc3339(&commit.3)
                .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|_| commit.3.clone());

            layout.row_list(
                &commit.0[..8.min(commit.0.len())],
                &format!("{}  {}", time_str, commit.1),
            );

            // Get files for this commit
            if let Ok(files) = repo.get_commit_files(&commit.0) {
                for file in files {
                    layout.row_list(
                        "  └─",
                        &format!("{} ({})", file.0, &file.1[..8.min(file.1.len())]),
                    );
                }
            }
        }

        layout.section_end();

        if commits.is_empty() {
            layout.info("No commits linked yet. Use 'mnem git-hook' to integrate with Git.");
        }

        Ok(())
    }
}
