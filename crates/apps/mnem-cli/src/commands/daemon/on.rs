use anyhow::Result;

use crate::ui::presentable::SimpleResponse;
use crate::ui::Presentable;

pub fn handle_on(_auto: bool, json: bool) -> Result<()> {
    use mnem_core::client;

    let response = match client::ensure_daemon() {
        Ok(true) => SimpleResponse {
            success: true,
            message: "mnem daemon started".to_string(),
            code: Some("DAEMON_STARTED".to_string()),
        },
        Ok(false) => SimpleResponse {
            success: true,
            message: "mnem daemon is already running".to_string(),
            code: Some("DAEMON_ALREADY_RUNNING".to_string()),
        },
        Err(e) => SimpleResponse {
            success: false,
            message: format!("Failed to start daemon: {}", e),
            code: Some("DAEMON_START_FAILED".to_string()),
        },
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}
