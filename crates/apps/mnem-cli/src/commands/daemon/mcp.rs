use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use mnem_core::client::DaemonClient;
use mnem_core::protocol::methods;

#[derive(Debug)]
pub struct McpCommand;

impl Command for McpCommand {
    fn name(&self) -> &str {
        "mcp"
    }

    fn usage(&self) -> &str {
        "<start|stop|status>"
    }

    fn description(&self) -> &str {
        "Manage MCP server (Model Context Protocol)"
    }

    fn group(&self) -> &str {
        "Daemon"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();

        let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

        match subcommand {
            "start" => {
                layout.header_dashboard("MCP SERVER");
                layout.info("Starting MCP server...");

                let mut client = DaemonClient::connect()?;
                let res = client.call(methods::MCP_START, serde_json::json!({}))?;

                if let Some(error) = res.get("error") {
                    layout.error(&format!("Failed: {}", error));
                } else {
                    let pid = res.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
                    layout.success_bright(&format!("MCP server started (PID: {})", pid));
                }
            }
            "stop" => {
                layout.header_dashboard("MCP SERVER");
                layout.info("Stopping MCP server...");

                let mut client = DaemonClient::connect()?;
                let res = client.call(methods::MCP_STOP, serde_json::json!({}))?;

                if let Some(error) = res.get("error") {
                    layout.error(&format!("Failed: {}", error));
                } else {
                    layout.success_bright("MCP server stopped");
                }
            }
            "status" => {
                layout.header_dashboard("MCP SERVER");

                let mut client = DaemonClient::connect()?;
                let res = client.call(methods::MCP_STATUS, serde_json::json!({}))?;

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
                    layout.row_property("PID", &pid.map(|p| p.to_string()).unwrap_or_default());
                    layout.row_property("Transport", transport);
                } else {
                    layout.error("MCP server is NOT running");
                    layout.info("Use 'mnem daemon mcp start' to start it");
                }
            }
            "" => {
                layout.header_dashboard("MCP SERVER");
                layout.usage("mnem daemon mcp <start|stop|status>");
                layout.empty();
                layout.item_simple("Commands:");
                layout.item_simple("  start   - Start MCP server");
                layout.item_simple("  stop    - Stop MCP server");
                layout.item_simple("  status  - Show MCP server status");
            }
            _ => {
                layout.error(&format!("Unknown subcommand: {}", subcommand));
                layout.usage("mnem daemon mcp <start|stop|status>");
            }
        }

        Ok(())
    }
}
