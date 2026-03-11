use anyhow::Result;
use serde_json::json;

use crate::ui::Layout;
use crate::ui::Renderable;
use crate::ui::presentable::SimpleResponse;

pub fn handle_mcp(subcommand: &str, json: bool) -> Result<()> {
    use mnem_core::client::DaemonClient;
    use mnem_core::protocol::methods;

    let layout = Layout::new();

    match subcommand {
        "start" => {
            if !json {
                layout.header_dashboard("MCP SERVER");
                layout.info("Starting MCP server...");
            }

            let response = match DaemonClient::connect() {
                Ok(mut client) => match client.call(methods::MCP_START, serde_json::json!({})) {
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
                },
                Err(_) => SimpleResponse {
                    success: false,
                    message: "Daemon is not running. Start it with 'mnem on'".to_string(),
                    code: Some("DAEMON_NOT_RUNNING".to_string()),
                },
            };

            if json {
                println!("{}", response.json()?);
            } else {
                response.text()?;
            }
        }

        "stop" => {
            if !json {
                layout.header_dashboard("MCP SERVER");
                layout.info("Stopping MCP server...");
            }

            let response = match DaemonClient::connect() {
                Ok(mut client) => match client.call(methods::MCP_STOP, serde_json::json!({})) {
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
                },
                Err(_) => SimpleResponse {
                    success: false,
                    message: "Daemon is not running".to_string(),
                    code: Some("DAEMON_NOT_RUNNING".to_string()),
                },
            };

            if json {
                println!("{}", response.json()?);
            } else {
                response.text()?;
            }
        }

        "status" => match DaemonClient::connect() {
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

                    if json {
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
                    if json {
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
            },
            Err(_) => {
                if json {
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
        },

        _ => {
            if json {
                println!(
                    "{}",
                    json!({
                        "success": false,
                        "error": format!("Unknown MCP subcommand: {}", subcommand),
                        "code": "UNKNOWN_SUBCOMMAND"
                    })
                );
            } else {
                layout.error(&format!("Unknown MCP subcommand: {}", subcommand));
                layout.info("Usage: mnem mcp <start|stop|status>");
            }
        }
    }

    Ok(())
}
