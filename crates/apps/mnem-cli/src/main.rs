// VERSION: v2-FIX-2024 - Testing binary replacement
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod commands;
mod theme;
mod ui;
mod ui_components;

use commands::Commands;

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

    if let Some(project_path) = cli.project {
        std::env::set_current_dir(project_path)?;
    }

    let json = cli.json;

    match cli.command {
        Some(Commands::On { auto }) => commands::handle_on(auto, json),
        Some(Commands::Off {}) => commands::handle_off(json),
        Some(Commands::Status {}) => commands::handle_status(json),
        Some(Commands::Track {
            list,
            remove,
            id,
            limit,
            page,
        }) => commands::handle_track(list, remove, id, limit, page, json),
        Some(Commands::H {
            file,
            limit,
            page,
            timeline,
            since,
            branch,
        }) => commands::handle_h(file, limit, page, timeline, since, branch, json),
        Some(Commands::R {
            file,
            version,
            list,
            undo,
            to,
            symbol,
            checkpoint,
            branch,
            limit,
            page,
        }) => commands::handle_r(
            file, version, list, undo, to, symbol, checkpoint, branch, limit, page, json,
        ),
        Some(Commands::S {
            query,
            file,
            limit,
            page,
            semantic,
        }) => commands::handle_s(query, file, limit, page, semantic, json),
        Some(Commands::Info { project }) => commands::handle_info(project, json),
        Some(Commands::Gc {
            keep,
            dry_run,
            aggressive,
        }) => commands::handle_gc(keep, dry_run, aggressive, json),
        Some(Commands::Config { get, set, reset }) => {
            commands::handle_config(get, set, reset, json)
        }
        Some(Commands::Git { commits, log, hook }) => {
            commands::handle_git(commits, log, hook, json)
        }
        Some(Commands::Uninstall { purge }) => commands::handle_uninstall(purge, json),
        Some(Commands::Update { check_only }) => commands::handle_update(check_only, json),
        Some(Commands::McpStart {}) => commands::handle_mcp("start", json),
        Some(Commands::McpStop {}) => commands::handle_mcp("stop", json),
        Some(Commands::McpStatus {}) => commands::handle_mcp("status", json),
        None => commands::handle_status(json),
    }
}
