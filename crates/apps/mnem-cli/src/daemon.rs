use anyhow::Result;
use mnem_core::client::DaemonClient;
use mnem_core::protocol::methods;
use serde_json::json;

/// Ensure the daemon process is running
/// Returns true if started, false if already running
pub fn ensure_running() -> Result<bool> {
    mnem_core::client::ensure_daemon()
        .map_err(|e| anyhow::anyhow!("Failed to ensure daemon running: {}", e))
}

/// Stop the daemon gracefully
pub fn stop() -> Result<()> {
    if let Ok(mut client) = DaemonClient::connect() {
        let _ = client.call(methods::SHUTDOWN, json!(null));
        let _ = client.call(methods::EXIT, json!(null));
    }
    Ok(())
}

/// Check if the daemon is responsive
pub fn is_running() -> bool {
    DaemonClient::connect().is_ok()
}

/// Get the daemon status
pub fn get_status() -> Result<mnem_core::protocol::StatusResponse> {
    let mut client = DaemonClient::connect()
        .map_err(|e| anyhow::anyhow!("Failed to connect to daemon: {}", e))?;
    let res = client
        .call(methods::DAEMON_GET_STATUS, json!({}))
        .map_err(|e| anyhow::anyhow!("Failed to get daemon status: {}", e))?;
    Ok(serde_json::from_value(res)?)
}
