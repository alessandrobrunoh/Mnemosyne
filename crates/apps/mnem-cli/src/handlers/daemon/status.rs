
use anyhow::Result;

use crate::ui::Layout;

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

pub fn handle_status() -> Result<()> {
    use mnem_core::client::DaemonClient;
    use mnem_core::protocol::methods;

    let layout = Layout::new();

    match DaemonClient::connect() {
        Ok(mut client) => {
            let res = client.call(methods::DAEMON_GET_STATUS, serde_json::json!({}))?;
            let status: mnem_core::protocol::StatusResponse = serde_json::from_value(res)?;

            layout.header_dashboard("MNEMOSYNE STATUS");

            layout.section_branch("da", "Daemon Status");
            layout.row_labeled("", "Running", "Active");
            layout.row_metric("", "Version", &status.version);
            layout.row_metric("", "Watched", &status.watched_projects.len().to_string());
            layout.row_metric("", "Uptime", &format_duration(status.uptime_secs));
            layout.row_metric(
                "",
                "Storage",
                &format!(
                    "{:.2} MB",
                    status.history_size_bytes as f64 / 1024.0 / 1024.0
                ),
            );
            layout.row_metric(
                "",
                "Avg Response",
                &format!("{:.2} ms", status.avg_response_time_ms),
            );
            layout.row_metric(
                "",
                "Avg Save",
                &format!("{:.2} ms", status.avg_save_time_ms),
            );
            layout.row_metric("", "Total Saves", &status.total_saves.to_string());
            layout.row_metric("", "Snapshots", &status.total_snapshots.to_string());
            layout.row_metric("", "Symbols", &status.total_symbols.to_string());
            layout.section_end();

            layout.empty();
            layout.badge_success("READY", "Mnemosyne is running");
        }
        Err(_) => {
            layout.header_dashboard("MNEMOSYNE STATUS");
            layout.section_branch("da", "Daemon Status");
            layout.row_labeled("", "Running", "Inactive");
            layout.section_end();
            layout.empty();
            layout.badge_info("TIP", "Run 'mnem on' to start the daemon");
            layout.info_bright("Run 'mnem on' to start the daemon.");
        }
    }
    Ok(())
}
