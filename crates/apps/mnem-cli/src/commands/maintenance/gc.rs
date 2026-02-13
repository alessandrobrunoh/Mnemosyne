use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{client::DaemonClient, protocol::methods, Repository};

#[derive(Debug)]
pub struct GcCommand;

impl Command for GcCommand {
    fn name(&self) -> &str {
        "gc"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Prune old snapshots and optimize storage"
    }

    fn group(&self) -> &str {
        "Maintenance"
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let layout = Layout::new();
        layout.section_start("gc", "Garbage Collection");

        if let Ok(mut client) = DaemonClient::connect() {
            let res = client.call(methods::MAINTENANCE_GC, serde_json::json!({}))?;
            let pruned = res["pruned"].to_string();
            layout.item_simple(&format!(
                "{} Successfully pruned {} orphan chunks.",
                "√".green(),
                pruned.bold()
            ));
        } else {
            let repo = Repository::init()?;
            let n = repo.run_gc()?;
            layout.item_simple(&format!(
                "{} Local cleanup complete: {} chunks pruned.",
                "√".green(),
                n.to_string().bold()
            ));
        }
        layout.section_end();
        Ok(())
    }
}
