use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::client::DaemonClient;
use mnem_core::protocol::methods;

#[derive(Debug)]
pub struct StatusCommand;

impl Command for StatusCommand {
    fn name(&self) -> &str {
        "status"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Show daemon status and information"
    }

    fn group(&self) -> &str {
        "Daemon"
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let layout = Layout::new();

        match DaemonClient::connect() {
            Ok(mut client) => {
                let res = client.call(methods::DAEMON_GET_STATUS, serde_json::json!({}))?;
                let status: mnem_core::protocol::StatusResponse = serde_json::from_value(res)?;

                layout.section_start("st", "Daemon Status");

                // Format latency display
                let latency_display = if status.avg_response_time_ms < 1.0 {
                    format!("{:.2}µs", status.avg_response_time_ms * 1000.0)
                } else {
                    format!("{:.3}ms", status.avg_response_time_ms)
                };

                // Display basic info
                let info = [
                    ("Running", "√"),
                    ("Version", &status.version),
                    ("Uptime", &format!("{}s", status.uptime_secs)),
                    ("Avg Latency", &latency_display),
                ];

                for (key, val) in info {
                    layout.row_property(key, val);
                }

                // Display storage info
                let hist_mb = status.history_size_bytes as f64 / 1024.0 / 1024.0;
                let total_mb = status.total_size_bytes as f64 / 1024.0 / 1024.0;
                let storage = format!("{:.2} MB (history) / {:.2} MB (total)", hist_mb, total_mb);
                layout.row_property("Storage", &storage);

                // Display watched projects
                if !status.watched_projects.is_empty() {
                    layout.empty();
                    layout.item_simple("Watching:");
                    for p in &status.watched_projects {
                        layout.item_simple(&format!("  √  {}", p));
                    }
                }

                layout.section_end();
            }
            Err(_) => {
                layout.error("Daemon is NOT running");
            }
        }

        Ok(())
    }
}
