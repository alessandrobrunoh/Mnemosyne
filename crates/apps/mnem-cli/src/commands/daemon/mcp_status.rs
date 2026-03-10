use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Presentable};

/// Show MCP server status
#[derive(Args, Clone, Debug)]
pub struct McpStatusCommand;

impl CommandStrategy for McpStatusCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        let layout = Layout::new();

        match mnem_core::client::DaemonClient::connect() {
            Ok(mut client) => {
                match client.call(mnem_core::protocol::methods::MCP_STATUS, json!({})) {
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

                        if global_opts.json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&json!({
                                    "success": true,
                                    "running": running,
                                    "pid": pid,
                                    "transport": transport
                                }))?
                            );
                        } else {
                            layout.header_dashboard("MCP SERVER");
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
                    }
                    Err(e) => {
                        if global_opts.json {
                            println!(
                                "{}",
                                json!({
                                    "success": false,
                                    "error": format!("Failed to get MCP status: {}", e),
                                    "code": "MCP_STATUS_ERROR"
                                })
                            );
                        } else {
                            layout.header_dashboard("MCP SERVER");
                            layout.error(&format!("Failed to get MCP status: {}", e));
                        }
                    }
                }
            }
            Err(_) => {
                if global_opts.json {
                    println!(
                        "{}",
                        json!({
                            "success": false,
                            "error": "Daemon is not running",
                            "code": "DAEMON_NOT_RUNNING"
                        })
                    );
                } else {
                    layout.header_dashboard("MCP SERVER");
                    layout.error("Daemon is not running");
                }
            }
        }

        Ok(())
    }
}
