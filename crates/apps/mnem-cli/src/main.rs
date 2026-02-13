use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod cp;
mod diff;
mod gc;
mod git;
mod history_new;
mod info;
mod off;
mod on;
mod open;
mod refs;
mod restore;
mod search;
mod stats;
mod status;
mod symbols;
mod timeline;
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
    #[command(about = "Show diff between versions")]
    D {
        file: Option<String>,
        #[arg(long)]
        from: Option<usize>,
        #[arg(long)]
        to: Option<usize>,
    },
    #[command(about = "Open file in IDE")]
    Open {
        file: Option<String>,
        #[arg(long, short)]
        at: Option<usize>,
        #[arg(long)]
        checkpoint: Option<String>,
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
    #[command(about = "Manage checkpoints")]
    Cp {
        message: Option<String>,
        #[arg(long, short)]
        list: bool,
        id: Option<String>,
        #[arg(long)]
        info: bool,
        #[arg(long)]
        remove: bool,
        #[arg(long)]
        restore: bool,
    },
    #[command(about = "Git integration")]
    Git {
        #[arg(long)]
        commits: bool,
        #[arg(long)]
        log: bool,
        #[arg(long)]
        hook: bool,
    },
    #[command(about = "Show symbols in file")]
    Symbols {
        file: Option<String>,
        #[arg(long, short)]
        search: Option<String>,
        #[arg(long)]
        modified: bool,
    },
    #[command(about = "Show symbol timeline")]
    Timeline {
        symbol: Option<String>,
        #[arg(long, short)]
        diff: bool,
    },
    #[command(about = "Find symbol references")]
    Refs {
        symbol: Option<String>,
        #[arg(long)]
        since: Option<String>,
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
        Some(Commands::D { file, from, to }) => diff::handle_d(file, from, to),
        Some(Commands::Open {
            file,
            at,
            checkpoint,
        }) => open::handle_open(file, at, checkpoint),
        Some(Commands::S {
            query,
            file,
            limit,
            semantic,
        }) => search::handle_s(query, file, limit, semantic),
        Some(Commands::Cp {
            message,
            list,
            id,
            info,
            remove,
            restore,
        }) => cp::handle_cp(message, list, id, info, remove, restore),
        Some(Commands::Git { commits, log, hook }) => git::handle_git(commits, log, hook),
        Some(Commands::Symbols {
            file,
            search,
            modified,
        }) => symbols::handle_symbols(file, search, modified),
        Some(Commands::Timeline { symbol, diff }) => timeline::handle_timeline(symbol, diff),
        Some(Commands::Refs { symbol, since }) => refs::handle_refs(symbol, since),
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
