use anyhow::Result;

use crate::ui::Layout;

pub fn handle_mcp(subcommand: &str) -> Result<()> {
    use mnem_core::client::DaemonClient;
    use mnem_core::protocol::methods;

    let layout = Layout::new();

    match subcommand {
        "start" => {
            layout.header_dashboard("MCP SERVER");
            layout.info("Starting MCP server...");

            match DaemonClient::connect() {
                Ok(mut client) => match client.call(methods::MCP_START, serde_json::json!({})) {
                    Ok(res) => {
                        if let Some(error) = res.get("error") {
                            layout.error(&format!("Failed: {}", error));
                        } else {
                            let pid = res.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                            layout.success_bright(&format!("MCP server started (PID: {})", pid));
                        }
                    }
                    Err(e) => {
                        layout.error(&format!("Failed to start MCP: {}", e));
                    }
                },
                Err(_) => {
                    layout.error("Daemon is not running. Start it with 'mnem on'");
                }
            }
        }

        "stop" => {
            layout.header_dashboard("MCP SERVER");
            layout.info("Stopping MCP server...");

            match DaemonClient::connect() {
                Ok(mut client) => match client.call(methods::MCP_STOP, serde_json::json!({})) {
                    Ok(res) => {
                        if let Some(error) = res.get("error") {
                            layout.error(&format!("Failed: {}", error));
                        } else {
                            layout.success_bright("MCP server stopped");
                        }
                    }
                    Err(e) => {
                        layout.error(&format!("Failed to stop MCP: {}", e));
                    }
                },
                Err(_) => {
                    layout.error("Daemon is not running");
                }
            }
        }

        "status" => {
            layout.header_dashboard("MCP SERVER");

            match DaemonClient::connect() {
                Ok(mut client) => match client.call(methods::MCP_STATUS, serde_json::json!({})) {
                    Ok(res) => {
                        let running = res
                            .get("running")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let pid = res.get("pid").and_then(|v| v.as_u64());
                        let transport = res
                            .get("transport")
                            .and_then(|v| v.as_str())
                            .unwrap_or("stdio");

                        if running {
                            layout.success_bright("MCP server is RUNNING");
                            layout.row_property(
                                "PID",
                                &pid.map(|p| p.to_string()).unwrap_or_default(),
                            );
                            layout.row_property("Transport", transport);
                        } else {
                            layout.error("MCP server is NOT running");
                            layout.info("Use 'mnem mcp start' to start it");
                        }
                    }
                    Err(e) => {
                        layout.error(&format!("Failed to get MCP status: {}", e));
                    }
                },
                Err(_) => {
                    layout.error("Daemon is not running");
                }
            }
        }

        _ => {
            layout.error(&format!("Unknown MCP subcommand: {}", subcommand));
            layout.info("Usage: mnem mcp <start|stop|status>");
        }
    }

    Ok(())
}
