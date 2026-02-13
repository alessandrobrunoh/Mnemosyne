use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod gc;
mod history_new;
mod info;
mod off;
mod on;
mod restore;
mod search;
mod status;
mod track;

mod theme;
mod ui;
mod ui_components;

#[derive(Parser)]
#[command(name = "mnem")]
#[command(version = "0.2.0")]
#[command(about = "Mnemosyne - Local history companion for developers", long_about = None)]
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
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the mnem daemon")]
    On {
        #[arg(long)]
        auto: bool,
    },
    #[command(about = "Stop the mnem daemon")]
    Off,
    #[command(about = "Show daemon and project status")]
    Status,
    #[command(about = "Track or list tracked projects")]
    Track {
        #[arg(long)]
        list: bool,
        #[arg(long, short)]
        remove: bool,
        #[arg(global = true)]
        id: Option<String>,
    },
    #[command(about = "Show file history")]
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
    #[command(about = "Restore file to previous version")]
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
    #[command(about = "Search code in history")]
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
    #[command(about = "Manage configuration")]
    Config {
        #[arg(long, short)]
        get: Option<String>,
        #[arg(long, short)]
        set: Option<String>,
        #[arg(long)]
        reset: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::On { auto }) => on::handle_on(auto),
        Some(Commands::Off) => off::handle_off(),
        Some(Commands::Status) => status::handle_status(),
        Some(Commands::Track { list, remove, id }) => track::handle_track(list, remove, id),
        Some(Commands::H {
            file,
            limit,
            timeline,
            since,
            branch,
        }) => history_new::handle_h(file, limit, timeline, since, branch),
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
        }) => restore::handle_r(
            file, version, list, undo, to, symbol, checkpoint, branch, limit,
        ),
        Some(Commands::S {
            query,
            file,
            limit,
            semantic,
        }) => search::handle_s(query, file, limit, semantic),
        Some(Commands::Info { project }) => info::handle_info(project),
        Some(Commands::Gc {
            keep,
            dry_run,
            aggressive,
        }) => gc::handle_gc(keep, dry_run, aggressive),
        Some(Commands::Config { get, set, reset }) => config::handle_config(get, set, reset),
        None => status::handle_status(),
    }
}
