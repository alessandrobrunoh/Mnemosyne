use anyhow::Result;
use mnem_core::{client::DaemonClient, protocol::methods};

use crate::ui::presentable::SimpleResponse;
use crate::ui::Presentable;

pub fn handle_gc(keep: Option<usize>, dry_run: bool, _aggressive: bool, json: bool) -> Result<()> {
    use crate::ui::Layout;
    use crossterm::style::Stylize;

    let layout = Layout::new();

    if dry_run {
        if json {
            println!("{}", serde_json::json!({ "success": true, "dry_run": true, "message": "Dry run - no changes made" }));
        } else {
            layout.section_start("gc", "Garbage Collection");
            layout.item_simple("Dry run - no changes will be made");
            layout.section_end();
        }
        return Ok(());
    }

    let response = match DaemonClient::connect() {
        Ok(mut client) => {
            let params = serde_json::json!({
                "keep_days": keep,
            });

            match client.call(methods::MAINTENANCE_GC, params) {
                Ok(res) => {
                    let pruned = res["pruned"].as_u64().unwrap_or(0);
                    SimpleResponse {
                        success: true,
                        message: format!("Successfully pruned {} orphan chunks.", pruned),
                        code: Some("GC_SUCCESS".to_string()),
                    }
                }
                Err(e) => SimpleResponse {
                    success: false,
                    message: format!("GC failed: {}", e),
                    code: Some("GC_FAILED".to_string()),
                },
            }
        }
        Err(_) => SimpleResponse {
            success: false,
            message: "Daemon is not running. Start it with 'mnem on'".to_string(),
            code: Some("DAEMON_NOT_RUNNING".to_string()),
        },
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        layout.section_start("gc", "Garbage Collection");
        if response.success {
            layout.item_simple(&format!(
                "{} {}",
                "√".green(),
                response.message.bold()
            ));
        } else {
            layout.error(&response.message);
        }
        layout.section_end();
    }

    Ok(())
}
