// VERSION: v2-FIX-2024 - Testing binary replacement
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod commands;
mod theme;
mod ui;

use commands::Commands;
use commands::common::{CommandStrategy, GlobalOptions};

#[derive(Parser)]
#[command(name = "mnem")]
#[command(version)]
#[command(about = "Mnemosyne - Local history companion", long_about = None)]
#[command(styles = styles())]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, global = true)]
    project: Option<PathBuf>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,

    #[arg(short, long, visible_alias = "j", global = true)]
    json: bool,
}

fn styles() -> clap::builder::Styles {
    use clap::builder::styling::{AnsiColor, Effects, Styles};
    Styles::styled()
        .header(AnsiColor::Magenta.on_default() | Effects::BOLD)
        .usage(AnsiColor::Magenta.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Yellow.on_default())
}

fn main() -> Result<()> {
    std::fs::write("/tmp/mnem_debug.log", "DEBUG: main() called\n").ok();
    let cli = Cli::parse();
    std::fs::write("/tmp/mnem_debug.log", "DEBUG: CLI parsed\n").ok();

    if let Some(ref project_path) = cli.project {
        std::env::set_current_dir(project_path)?;
    }

    let global_opts = GlobalOptions {
        project: cli.project,
        json: cli.json,
    };

    match cli.command {
        Some(cmd) => cmd.execute(&global_opts),
        None => {
            // Default to showing status when no command is provided
            commands::daemon::StatusCommand.execute(&global_opts)
        }
    }
}
