use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::presentable::SimpleResponse;
use crate::ui::{Layout, Presentable};

/// Start the MCP server
#[derive(Args, Clone, Debug)]
pub struct McpStartCommand;

impl CommandStrategy for McpStartCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        let layout = Layout::new();

        if !global_opts.json {
            layout.header_dashboard("MCP SERVER");
            layout.info("Starting MCP server...");
        }

        let response = match mnem_core::client::DaemonClient::connect() {
            Ok(mut client) => {
                match client.call(mnem_core::protocol::methods::MCP_START, json!({})) {
                    Ok(res) => {
                        if let Some(error) = res.get("error") {
                            SimpleResponse {
                                success: false,
                                message: format!("Failed: {}", error),
                                code: Some("MCP_START_FAILED".to_string()),
                            }
                        } else {
                            let pid = res.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                            SimpleResponse {
                                success: true,
                                message: format!("MCP server started (PID: {})", pid),
                                code: Some("MCP_STARTED".to_string()),
                            }
                        }
                    }
                    Err(e) => SimpleResponse {
                        success: false,
                        message: format!("Failed to start MCP: {}", e),
                        code: Some("MCP_RPC_ERROR".to_string()),
                    },
                }
            }
            Err(_) => SimpleResponse {
                success: false,
                message: "Daemon is not running. Start it with 'mnem on'".to_string(),
                code: Some("DAEMON_NOT_RUNNING".to_string()),
            },
        };

        if global_opts.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&response.render_json()?)?
            );
        } else {
            response.render_tui()?;
        }

        Ok(())
    }
}
