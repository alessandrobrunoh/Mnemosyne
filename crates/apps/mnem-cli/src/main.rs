use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod handlers;
mod theme;
mod ui;
mod ui_components;

#[derive(Parser)]
#[command(name = "mnem")]
#[command(version = "0.1.3")]
#[command(about = "Mnemosyne - Local history companion", long_about = None)]
#[command(styles = styles())]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, global = true)]
    project: Option<PathBuf>,
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
    #[command(about = "Start daemon")]
    On {
        #[arg(long)]
        auto: bool,
    },
    #[command(about = "Stop daemon")]
    Off {},
    #[command(about = "Show status")]
    Status {},
    #[command(about = "Track project")]
    Track {
        #[arg(long)]
        list: bool,
        #[arg(long, short)]
        remove: bool,
        #[arg(global = true)]
        id: Option<String>,
    },
    #[command(about = "View history")]
    H {
        file: Option<String>,
        #[arg(long, short)]
        limit: Option<usize>,
        #[arg(long)]
        timeline: bool,
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(about = "Restore file")]
    R {
        file: Option<String>,
        version: Option<usize>,
        #[arg(long, short)]
        list: bool,
        #[arg(long)]
        undo: bool,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        checkpoint: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long, short)]
        limit: Option<usize>,
    },
    #[command(about = "Search history")]
    S {
        query: Option<String>,
        #[arg(long, short)]
        file: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        semantic: bool,
    },
    #[command(about = "Show project info")]
    Info { project: Option<String> },
    #[command(about = "Garbage collection")]
    Gc {
        #[arg(long)]
        keep: Option<usize>,
        #[arg(long, short)]
        dry_run: bool,
        #[arg(long)]
        aggressive: bool,
    },
    #[command(about = "Manage config")]
    Config {
        #[arg(long, short)]
        get: Option<String>,
        #[arg(long)]
        set: Option<String>,
        #[arg(long)]
        reset: bool,
    },
    #[command(about = "Uninstall mnem")]
    Uninstall {},
    #[command(about = "Check for updates and update")]
    Update {
        #[arg(long)]
        check_only: bool,
    },
    #[command(about = "Start MCP server")]
    McpStart {},
    #[command(about = "Stop MCP server")]
    McpStop {},
    #[command(about = "Show MCP server status")]
    McpStatus {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(project_path) = cli.project {
        std::env::set_current_dir(project_path)?;
    }

    match cli.command {
        Some(Commands::On { auto }) => handlers::handle_on(auto),
        Some(Commands::Off {}) => handlers::handle_off(),
        Some(Commands::Status {}) => handlers::handle_status(),
        Some(Commands::Track { list, remove, id }) => handlers::handle_track(list, remove, id),
        Some(Commands::H {
            file,
            limit,
            timeline,
            since,
            branch,
        }) => handlers::handle_h(file, limit, timeline, since, branch),
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
        }) => handlers::handle_r(
            file, version, list, undo, to, symbol, checkpoint, branch, limit,
        ),
        Some(Commands::S {
            query,
            file,
            limit,
            semantic,
        }) => handlers::handle_s(query, file, limit, semantic),
        Some(Commands::Info { project }) => handlers::handle_info(project),
        Some(Commands::Gc {
            keep,
            dry_run,
            aggressive,
        }) => handlers::handle_gc(keep, dry_run, aggressive),
        Some(Commands::Config { get, set, reset }) => handlers::handle_config(get, set, reset),
        Some(Commands::Uninstall {}) => handlers::handle_uninstall(),
        Some(Commands::Update { check_only }) => handlers::handle_update(check_only),
        Some(Commands::McpStart {}) => handlers::handle_mcp("start"),
        Some(Commands::McpStop {}) => handlers::handle_mcp("stop"),
        Some(Commands::McpStatus {}) => handlers::handle_mcp("status"),
        None => handlers::handle_status(),
    }
}
