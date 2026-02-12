use crate::ui::ButlerLayout;
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::client::{self, DaemonClient};
use mnem_core::protocol::methods;

pub fn start(args: &[String]) -> Result<()> {
    let restart = args.iter().any(|a| a == "--restart" || a == "-r");
    if restart {
        if let Ok(mut client) = DaemonClient::connect() {
            let _ = client.call(methods::SHUTDOWN, serde_json::json!(null));
            let _ = client.call(methods::EXIT, serde_json::json!(null));
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    match client::ensure_daemon() {
        Ok(true) => println!("{} mnemd daemon started.", "✓".green()),
        Ok(false) => {
            if restart {
                println!("{} mnemd daemon restarted.", "✓".green());
            } else {
                println!("{} mnemd daemon is already running.", "✓".green());
            }
        }
        Err(e) => eprintln!("{} Failed to start daemon: {}", "✘".red(), e),
    }
    Ok(())
}

pub fn stop() -> Result<()> {
    if let Ok(mut client) = DaemonClient::connect() {
        let _ = client.call(methods::SHUTDOWN, serde_json::json!(null));
        let _ = client.call(methods::EXIT, serde_json::json!(null));
        println!("{} Daemon shutdown requested.", "✓".green());
    } else {
        println!("{} Daemon is not running.", "○".red());
    }
    Ok(())
}

pub fn status() -> Result<()> {
    match DaemonClient::connect() {
        Ok(mut client) => {
            let res = client.call(methods::DAEMON_GET_STATUS, serde_json::json!({}))?;
            let status: mnem_core::protocol::StatusResponse = serde_json::from_value(res)?;

            ButlerLayout::section_start("st", "Daemon Status");

            let latency_display = if status.avg_response_time_ms < 1.0 {
                format!("{:.2}µs", status.avg_response_time_ms * 1000.0)
            } else {
                format!("{:.3}ms", status.avg_response_time_ms)
            };

            let info = [
                ("Running", "√".green().to_string()),
                ("Version", status.version.dark_grey().to_string()),
                ("Uptime", format!("{}s", status.uptime_secs)),
                ("Avg Latency", latency_display),
            ];

            for (key, val) in info {
                let content = format!("{: <15} {}", key.white().dim(), val);
                ButlerLayout::row_list("•", &content);
            }

            let hist_mb = status.history_size_bytes as f64 / 1024.0 / 1024.0;
            let total_mb = status.total_size_bytes as f64 / 1024.0 / 1024.0;
            let storage = format!("{:.2} MB (history) / {:.2} MB (total)", hist_mb, total_mb);
            ButlerLayout::row_list(
                "•",
                &format!("{: <15} {}", "Storage".white().dim(), storage),
            );

            if !status.watched_projects.is_empty() {
                ButlerLayout::item_simple("");
                ButlerLayout::item_simple(&"Watching:".bold().cyan().to_string());
                for p in status.watched_projects {
                    ButlerLayout::item_simple(&format!("  {}  {}", "√".green(), p));
                }
            }
            ButlerLayout::section_end();
        }
        Err(_) => {
            println!("{} {}", "○".red(), "Daemon is NOT running".bold());
        }
    }
    Ok(())
}
