use anyhow::Result;
use clap::Args;
use mnem_core::protocol::StatusResponse;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Renderable};

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

impl Renderable for StatusResponse {
    fn text(&self) -> Result<()> {
        let layout = Layout::new();
        let theme = layout.theme();

        layout.graph_branch_start("daemon: Mnemosyne");

        // 1. Core Info
        layout.graph_node(
            &self.version,
            "VERSION",
            true,
            "running",
            None,
            theme.success_bright,
        );

        let uptime = format_duration(self.uptime_secs);
        layout.graph_node(&uptime, "UPTIME", false, "active", None, theme.text_dim);

        layout.graph_connector();

        // 2. Performance
        layout.graph_block_header("⚡", "performance", theme.timeline_purple);
        layout.graph_node(
            &format!("{:.2} ms", self.avg_response_time_ms),
            "AVG RESPONSE",
            false,
            "stable",
            None,
            theme.text_dim,
        );
        layout.graph_node(
            &format!("{:.2} ms", self.avg_save_time_ms),
            "AVG SAVE",
            false,
            "stable",
            None,
            theme.text_dim,
        );

        layout.graph_connector();

        // 3. Storage
        layout.graph_block_header("💾", "storage", theme.timeline_cyan);
        layout.graph_node(
            &format!("{:.2} MB", self.history_size_bytes as f64 / 1024.0 / 1024.0),
            "DB SIZE",
            false,
            "active",
            None,
            theme.text_dim,
        );
        layout.graph_node(
            &self.total_snapshots.to_string(),
            "SNAPSHOTS",
            false,
            "indexed",
            None,
            theme.text_dim,
        );

        layout.graph_branch_end();
        Ok(())
    }
}

/// Show daemon status and performance metrics
#[derive(Args, Clone, Debug)]
#[command(alias = "s")]
pub struct StatusCommand;

impl CommandStrategy for StatusCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        use mnem_core::client::DaemonClient;
        use mnem_core::protocol::methods;

        match DaemonClient::connect() {
            Ok(mut client) => {
                let res = client.call(methods::DAEMON_GET_STATUS, serde_json::json!({}))?;
                let status: StatusResponse = serde_json::from_value(res)?;

                if global_opts.json {
                    println!("{}", status.json()?);
                } else {
                    status.text()?;
                }
            }
            Err(_) => {
                if global_opts.json {
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
                    layout.graph_branch_start("daemon: Mnemosyne");
                    layout.graph_node(
                        "OFF",
                        "STATUS",
                        false,
                        "stopped",
                        None,
                        crossterm::style::Color::Red,
                    );
                    layout.graph_branch_end();
                    layout.empty();
                    layout.badge_info("TIP", "Run 'mnem on' to start the daemon");
                }
            }
        }
        Ok(())
    }
}
