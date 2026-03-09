use anyhow::Result;

use crate::ui::presentable::SimpleResponse;
use crate::ui::Presentable;

pub fn handle_off(json: bool) -> Result<()> {
    use mnem_core::client::DaemonClient;
    use mnem_core::protocol::methods;

    let response = match DaemonClient::connect() {
        Ok(mut client) => {
            let _ = client.call(methods::SHUTDOWN, serde_json::json!(null));
            let _ = client.call(methods::EXIT, serde_json::json!(null));
            SimpleResponse {
                success: true,
                message: "mnem daemon stopped".to_string(),
                code: Some("DAEMON_STOPPED".to_string()),
            }
        }
        Err(_) => SimpleResponse {
            success: true, // It's technically successful if it's already not running
            message: "mnem daemon is not running".to_string(),
            code: Some("DAEMON_NOT_RUNNING".to_string()),
        },
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}
