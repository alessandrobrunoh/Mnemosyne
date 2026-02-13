use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{client::DaemonClient, protocol::methods};

#[derive(Debug)]
pub struct TimelineCommand;

impl Command for TimelineCommand {
    fn name(&self) -> &str {
        "timeline"
    }

    fn usage(&self) -> &str {
        "<symbol_name>"
    }

    fn description(&self) -> &str {
        "Show semantic evolution timeline of a symbol"
    }

    fn group(&self) -> &str {
        "Files"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        if args.len() < 3 {
            layout.usage(self.name(), self.usage());
            return Ok(());
        }
        let symbol_name = &args[2];

        let mut client = DaemonClient::connect()?;
        let res = client.call(
            methods::SYMBOL_GET_SEMANTIC_HISTORY,
            serde_json::json!({ "symbol_name": symbol_name }),
        )?;

        let resp: mnem_core::protocol::SemanticHistoryResponse = serde_json::from_value(res)?;

        layout.header("SEMANTIC TIMELINE");
        layout.section_start("sm", symbol_name);

        if resp.deltas.is_empty() {
            layout.item_simple("No semantic history found for this symbol.");
        } else {
            for delta in resp.deltas {
                let kind_styled = match delta.kind {
                    mnem_core::models::RecordKind::Added => "CREATED".green().bold(),
                    mnem_core::models::RecordKind::Modified => "MODIFIED".blue().bold(),
                    mnem_core::models::RecordKind::Deleted => "DELETED".red().bold(),
                    mnem_core::models::RecordKind::Renamed => "RENAMED".yellow().bold(),
                };

                let name_info = if let Some(new_name) = delta.new_name {
                    format!("{} → {}", delta.symbol_name, new_name)
                } else {
                    delta.symbol_name.clone()
                };

                let hash_short = &delta.structural_hash[..8.min(delta.structural_hash.len())];
                let content = format!(
                    "{: <10}  {: <20}  {}",
                    kind_styled,
                    name_info.white(),
                    hash_short.dark_grey()
                );
                layout.item_simple(&content);
            }
        }

        layout.section_end();
        Ok(())
    }
}
