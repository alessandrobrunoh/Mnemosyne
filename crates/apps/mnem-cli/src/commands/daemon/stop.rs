use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::client::DaemonClient;
use mnem_core::protocol::methods;

#[derive(Debug)]
pub struct StopCommand;

impl Command for StopCommand {
    fn name(&self) -> &str {
        "stop"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Stop the mnemd daemon gracefully"
    }

    fn group(&self) -> &str {
        "Daemon"
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let layout = Layout::new();

        if let Ok(mut client) = DaemonClient::connect() {
            let _ = client.call(methods::SHUTDOWN, serde_json::json!(null));
            let _ = client.call(methods::EXIT, serde_json::json!(null));
            layout.success("Daemon shutdown requested.");
        } else {
            layout.warning("Daemon is not running.");
        }

        Ok(())
    }
}
