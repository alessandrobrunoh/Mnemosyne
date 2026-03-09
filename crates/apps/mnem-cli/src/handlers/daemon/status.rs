use anyhow::Result;
use mnem_core::protocol::StatusResponse;
use serde_json::Value;

use crate::ui::{Layout, Presentable};

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

impl Presentable for StatusResponse {
    fn render_tui(&self) -> Result<()> {
        let layout = Layout::new();
        layout.header_dashboard("MNEMOSYNE STATUS");

        layout.section_branch("da", "Daemon Status");
        layout.row_labeled("", "Running", "Active");
        layout.row_metric("", "Version", &self.version);
        layout.row_metric("", "Watched", &self.watched_projects.len().to_string());
        layout.row_metric("", "Uptime", &format_duration(self.uptime_secs));
        layout.row_metric(
            "",
            "Storage",
            &format!(
                "{:.2} MB",
                self.history_size_bytes as f64 / 1024.0 / 1024.0
            ),
        );
        layout.row_metric(
            "",
            "Avg Response",
            &format!("{:.2} ms", self.avg_response_time_ms),
        );
        layout.row_metric(
            "",
            "Avg Save",
            &format!("{:.2} ms", self.avg_save_time_ms),
        );
        layout.row_metric("", "Total Saves", &self.total_saves.to_string());
        layout.row_metric("", "Snapshots", &self.total_snapshots.to_string());
        layout.row_metric("", "Symbols", &self.total_symbols.to_string());
        layout.section_end();

        layout.empty();
        layout.badge_success("READY", "Mnemosyne is running");
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_status(json: bool) -> Result<()> {
    use mnem_core::client::DaemonClient;
    use mnem_core::protocol::methods;

    match DaemonClient::connect() {
        Ok(mut client) => {
            let res = client.call(methods::DAEMON_GET_STATUS, serde_json::json!({}))?;
            let status: StatusResponse = serde_json::from_value(res)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&status.render_json()?)?);
            } else {
                status.render_tui()?;
            }
        }
        Err(_) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": "Daemon is NOT running",
                        "code": "DAEMON_NOT_RUNNING"
                    })
                );
            } else {
                let layout = Layout::new();
                layout.header_dashboard("MNEMOSYNE STATUS");
                layout.section_branch("da", "Daemon Status");
                layout.row_labeled("", "Running", "Inactive");
                layout.section_end();
                layout.empty();
                layout.badge_info("TIP", "Run 'mnem on' to start the daemon");
                layout.info_bright("Run 'mnem on' to start the daemon.");
            }
        }
    }
    Ok(())
}
