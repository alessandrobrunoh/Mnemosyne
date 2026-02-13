use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::client;
use mnem_core::client::DaemonClient;
use mnem_core::protocol::methods;

#[derive(Debug)]
pub struct StartCommand;

impl Command for StartCommand {
    fn name(&self) -> &str {
        "start"
    }

    fn usage(&self) -> &str {
        "[--restart | -r]"
    }

    fn description(&self) -> &str {
        "Start the mnemd daemon"
    }

    fn group(&self) -> &str {
        "Daemon"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();

        let restart = args.iter().any(|a| a == "--restart" || a == "-r");
        if restart {
            if let Ok(mut client) = DaemonClient::connect() {
                let _ = client.call(methods::SHUTDOWN, serde_json::json!(null));
                let _ = client.call(methods::EXIT, serde_json::json!(null));
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        match client::ensure_daemon() {
            Ok(true) => {
                layout.success("mnemd daemon started.");
                Ok(())
            }
            Ok(false) => {
                if restart {
                    layout.success("mnemd daemon restarted.");
                } else {
                    layout.info("mnemd daemon is already running.");
                }
                Ok(())
            }
            Err(e) => {
                layout.error(&format!("Failed to start daemon: {}", e));
                Err(e.into())
            }
        }
    }
}
