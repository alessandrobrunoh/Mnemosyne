use anyhow::Result;

use crate::ui::Layout;

pub fn handle_off() -> Result<()> {
    use mnem_core::client::DaemonClient;
    use mnem_core::protocol::methods;

    let layout = Layout::new();

    match DaemonClient::connect() {
        Ok(mut client) => {
            let _ = client.call(methods::SHUTDOWN, serde_json::json!(null));
            let _ = client.call(methods::EXIT, serde_json::json!(null));
            layout.header_dashboard("DAEMON");
            layout.success_bright("✓ mnem daemon stopped");
            layout.empty();
            layout.badge_info("STOPPED", "Daemon has been shut down");
        }
        Err(_) => {
            layout.header_dashboard("DAEMON");
            layout.info_bright("● mnem daemon is not running");
            layout.empty();
            layout.badge_info("INFO", "No daemon was running");
        }
    }
    Ok(())
}
