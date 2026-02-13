use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct GitEventCommand;

impl Command for GitEventCommand {
    fn name(&self) -> &str {
        "git-event"
    }

    fn usage(&self) -> &str {
        "<hash> <message> <author> <timestamp>"
    }

    fn description(&self) -> &str {
        "Internal command: link a Git commit to Mnemosyne snapshots"
    }

    fn group(&self) -> &str {
        "Git Integration"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        if args.len() < 6 {
            let layout = Layout::new();
            layout.usage(self.name(), self.usage());
            std::process::exit(1);
        }

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

        let commit_hash = &args[2];
        let message = &args[3];
        let author = &args[4];
        let timestamp = &args[5];

        repo.insert_git_commit(commit_hash, message, author, timestamp)?;

        Ok(())
    }
}
