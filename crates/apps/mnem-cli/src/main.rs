// VERSION: v2-FIX-2024 - Testing binary replacement
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod handlers;
mod theme;
mod ui;
mod ui_components;

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

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start daemon", visible_alias = "on")]
    On {
        #[arg(short, long)]
        auto: bool,
    },
    #[command(about = "Stop daemon", visible_alias = "off")]
    Off {},
    #[command(about = "Show status", visible_alias = "st")]
    Status {},
    #[command(about = "Track project", visible_alias = "track")]
    Track {
        #[arg(short, long)]
        list: bool,
        #[arg(short, long)]
        remove: bool,
        #[arg(short, long)]
        id: Option<String>,
    },
    #[command(about = "View history", visible_alias = "history")]
    H {
        file: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
        #[arg(short = 'P', long, default_value = "1")]
        page: usize,
        #[arg(short, long)]
        timeline: bool,
        #[arg(short, long)]
        since: Option<String>,
        #[arg(short, long)]
        branch: Option<String>,
    },
    #[command(about = "Restore file", visible_alias = "restore")]
    R {
        file: Option<String>,
        version: Option<usize>,
        #[arg(short, long)]
        list: bool,
        #[arg(short, long)]
        undo: bool,
        #[arg(short, long)]
        to: Option<String>,
        #[arg(short, long)]
        symbol: Option<String>,
        #[arg(short, long)]
        checkpoint: Option<String>,
        #[arg(short, long)]
        branch: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
        #[arg(short = 'P', long, default_value = "1")]
        page: usize,
    },
    #[command(about = "Search history", visible_alias = "search")]
    S {
        query: Option<String>,
        #[arg(short, long)]
        file: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
        #[arg(short = 'P', long, default_value = "1")]
        page: usize,
        #[arg(short, long)]
        semantic: bool,
    },
    #[command(about = "Show project info", visible_alias = "info")]
    Info { project: Option<String> },
    #[command(about = "Garbage collection", visible_alias = "cleanup")]
    Gc {
        #[arg(short, long)]
        keep: Option<usize>,
        #[arg(short, long)]
        dry_run: bool,
        #[arg(short, long)]
        aggressive: bool,
    },
    #[command(about = "Manage config", visible_alias = "cfg")]
    Config {
        #[arg(short, long)]
        get: Option<String>,
        #[arg(short, long)]
        set: Option<String>,
        #[arg(short, long)]
        reset: bool,
    },
    #[command(about = "Git operations", visible_alias = "git")]
    Git {
        #[arg(short, long)]
        commits: bool,
        #[arg(short, long)]
        log: bool,
        #[arg(short, long)]
        hook: bool,
    },
    #[command(about = "Uninstall mnem", visible_alias = "remove")]
    Uninstall {
        #[arg(short, long)]
        purge: bool,
    },
    #[command(about = "Check for updates and update", visible_alias = "upgrade")]
    Update {
        #[arg(short, long)]
        check_only: bool,
    },
    #[command(about = "Start MCP server", visible_alias = "mcp-start")]
    McpStart {},
    #[command(about = "Stop MCP server", visible_alias = "mcp-stop")]
    McpStop {},
    #[command(about = "Show MCP server status", visible_alias = "mcp-status")]
    McpStatus {},
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
        Some(Commands::On { auto }) => handlers::handle_on(auto, json),
        Some(Commands::Off {}) => handlers::handle_off(json),
        Some(Commands::Status {}) => handlers::handle_status(json),
        Some(Commands::Track { list, remove, id }) => {
            handlers::handle_track(list, remove, id, json)
        }
        Some(Commands::H {
            file,
            limit,
            page,
            timeline,
            since,
            branch,
        }) => handlers::handle_h(file, limit, page, timeline, since, branch, json),
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
        }) => handlers::handle_r(
            file, version, list, undo, to, symbol, checkpoint, branch, limit, page, json,
        ),
        Some(Commands::S {
            query,
            file,
            limit,
            page,
            semantic,
        }) => handlers::handle_s(query, file, limit, page, semantic, json),
        Some(Commands::Info { project }) => handlers::handle_info(project, json),
        Some(Commands::Gc {
            keep,
            dry_run,
            aggressive,
        }) => handlers::handle_gc(keep, dry_run, aggressive, json),
        Some(Commands::Config { get, set, reset }) => {
            handlers::handle_config(get, set, reset, json)
        }
        Some(Commands::Git { commits, log, hook }) => {
            handlers::handle_git(commits, log, hook, json)
        }
        Some(Commands::Uninstall { purge }) => handlers::handle_uninstall(purge, json),
        Some(Commands::Update { check_only }) => handlers::handle_update(check_only, json),
        Some(Commands::McpStart {}) => handlers::handle_mcp("start", json),
        Some(Commands::McpStop {}) => handlers::handle_mcp("stop", json),
        Some(Commands::McpStatus {}) => handlers::handle_mcp("status", json),
        None => handlers::handle_status(json),
    }
}
