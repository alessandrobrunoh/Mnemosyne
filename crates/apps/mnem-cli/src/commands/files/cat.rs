use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::{client::DaemonClient, protocol::methods};

#[derive(Debug)]
pub struct CatCommand;

impl Command for CatCommand {
    fn name(&self) -> &str {
        "cat"
    }

    fn usage(&self) -> &str {
        "<hash>"
    }

    fn description(&self) -> &str {
        "Print the content of a specific snapshot to stdout"
    }

    fn group(&self) -> &str {
        "Files"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        if args.len() < 3 {
            layout.usage(self.name(), self.usage());
            return Ok(());
        }
        let hash = &args[2];

        let _ = mnem_core::client::ensure_daemon();

        let mut client = DaemonClient::connect()?;
        let res = client.call(
            methods::SNAPSHOT_GET,
            serde_json::json!({ "content_hash": hash }),
        )?;
        let content_res: serde_json::Value = serde_json::from_value(res)?;
        let content = content_res["content"].as_str().unwrap_or("");

        println!("{}", content);
        Ok(())
    }
}
