use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::presentable::SimpleResponse;
use crate::ui::{Layout, Presentable};

/// Stop the MCP server
#[derive(Args, Clone, Debug)]
pub struct McpStopCommand;

impl CommandStrategy for McpStopCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        let layout = Layout::new();

        if !global_opts.json {
            layout.header_dashboard("MCP SERVER");
            layout.info("Stopping MCP server...");
        }

        let response = match mnem_core::client::DaemonClient::connect() {
            Ok(mut client) => {
                match client.call(mnem_core::protocol::methods::MCP_STOP, json!({})) {
                    Ok(res) => {
                        if let Some(error) = res.get("error") {
                            SimpleResponse {
                                success: false,
                                message: format!("Failed: {}", error),
                                code: Some("MCP_STOP_FAILED".to_string()),
                            }
                        } else {
                            SimpleResponse {
                                success: true,
                                message: "MCP server stopped".to_string(),
                                code: Some("MCP_STOPPED".to_string()),
                            }
                        }
                    }
                    Err(e) => SimpleResponse {
                        success: false,
                        message: format!("Failed to stop MCP: {}", e),
                        code: Some("MCP_RPC_ERROR".to_string()),
                    },
                }
            }
            Err(_) => SimpleResponse {
                success: false,
                message: "Daemon is not running".to_string(),
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
