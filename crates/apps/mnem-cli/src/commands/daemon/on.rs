use anyhow::Result;
use clap::Args;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::Presentable;
use crate::ui::presentable::SimpleResponse;

/// Start the Mnemosyne daemon
#[derive(Args, Clone, Debug)]
pub struct OnCommand {
    /// Start daemon automatically (if supported)
    #[arg(short, long)]
    pub auto: bool,
}

impl CommandStrategy for OnCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        let response = match mnem_core::client::ensure_daemon() {
            Ok(true) => SimpleResponse {
                success: true,
                message: "mnem daemon started".to_string(),
                code: Some("DAEMON_STARTED".to_string()),
            },
            Ok(false) => SimpleResponse {
                success: true,
                message: "mnem daemon is already running".to_string(),
                code: Some("DAEMON_ALREADY_RUNNING".to_string()),
            },
            Err(e) => SimpleResponse {
                success: false,
                message: format!("Failed to start daemon: {}", e),
                code: Some("DAEMON_START_FAILED".to_string()),
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
