use crate::commands::Command;
use crate::ui::{Layout, TsHighlighter};
use crate::utils;
use anyhow::Result;
use mnem_core::{env::get_base_dir, Repository};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct LogCommand;

impl Command for LogCommand {
    fn name(&self) -> &str {
        "log"
    }

    fn usage(&self) -> &str {
        "<file_path> [--symbol <name>]"
    }

    fn description(&self) -> &str {
        "Show semantic history log for a file or specific symbol"
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

        // Check for --symbol flag
        if let Some(pos) = args.iter().position(|a| a == "--symbol" || a == "-s") {
            if let Some(symbol_name) = args.get(pos + 1) {
                return self.log_symbol(symbol_name, args);
            }
        }

        let file_path = &args[2];
        let common_args = utils::parse_common_args(args);

        let base_dir = get_base_dir()?;
        let cwd = env::current_dir()?;
        let repo = Repository::open(base_dir, cwd)?;

        let history = repo.get_history(file_path)?;
        let history: Vec<_> = history.into_iter().take(common_args.limit).collect();

        layout.header(&format!("HISTORY: {}", file_path));
        layout.section_start("log", "Semantic Snapshots");

        for (idx, snapshot) in history.iter().enumerate() {
            let styled_hash = &snapshot.content_hash[..8.min(snapshot.content_hash.len())];
            let time_str = chrono::DateTime::parse_from_rfc3339(&snapshot.timestamp)
                .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|_| snapshot.timestamp.clone());

            let content = format!("{}  {}", time_str, snapshot.content_hash);

            if idx == 0 {
                layout.row_snapshot_latest(styled_hash, &content);
            } else {
                layout.row_snapshot(styled_hash, &content);
            }
        }

        layout.section_end();
        Ok(())
    }
}

impl LogCommand {
    fn log_symbol(&self, symbol_name: &str, args: &[String]) -> Result<()> {
        let layout = Layout::new();
        let common_args = utils::parse_common_args(args);
        let base_dir = get_base_dir()?;
        let cwd = env::current_dir()?;
        let repo = Repository::open(base_dir, cwd)?;

        let history = repo.db.get_symbol_history(symbol_name)?;
        let history: Vec<_> = history.into_iter().take(common_args.limit).collect();

        layout.header(&format!("SYMBOL LOG: {}", symbol_name));
        layout.section_start("sym", "Semantic Evolution");

        for (snapshot, _symbol) in history {
            let styled_hash = &snapshot.content_hash[..8.min(snapshot.content_hash.len())];
            let time_str = chrono::DateTime::parse_from_rfc3339(&snapshot.timestamp)
                .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|_| snapshot.timestamp.clone());

            layout.row_snapshot(
                styled_hash,
                &format!("{}  {}", time_str, snapshot.file_path),
            );
        }

        layout.section_end();
        Ok(())
    }
}
