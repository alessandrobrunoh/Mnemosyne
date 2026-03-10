use anyhow::Result;
use clap::Args;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::Renderable;
use crate::ui::presentable::SimpleResponse;

/// Stop the Mnemosyne daemon
#[derive(Args, Clone, Debug)]
pub struct OffCommand;

impl CommandStrategy for OffCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
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

        if global_opts.json {
            println!("{}", response.json()?);
        } else {
            response.text()?;
        }

        Ok(())
    }
}
