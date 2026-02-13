use anyhow::Result;

use crate::ui::Layout;

pub fn handle_on(_auto: bool) -> Result<()> {
    use mnem_core::client;

    let layout = Layout::new();

    match client::ensure_daemon() {
        Ok(true) => {
            layout.header_dashboard("DAEMON");
            layout.success_bright("✓ mnem daemon started");
            layout.empty();
            layout.badge_success("READY", "Mnemosyne is now running");
        }
        Ok(false) => {
            layout.header_dashboard("DAEMON");
            layout.info_bright("● mnem daemon is already running");
            layout.empty();
            layout.badge_info("INFO", "Daemon was already active");
        }
        Err(e) => {
            layout.header_dashboard("DAEMON");
            layout.error_bright(&format!("✗ Failed to start daemon: {}", e));
        }
    }
    Ok(())
}
