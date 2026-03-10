use clap::Subcommand;

pub mod daemon;
pub mod files;
pub mod general;
pub mod maintenance;
pub mod workspace;

pub use daemon::handle_mcp;
pub use daemon::handle_off;
pub use daemon::handle_on;
pub use daemon::handle_status;
pub use files::handle_h;
pub use files::handle_info;
pub use files::handle_r;
pub use files::handle_s;
pub use general::handle_git;
pub use maintenance::handle_config;
pub use maintenance::handle_gc;
pub use maintenance::handle_uninstall;
pub use maintenance::handle_update;
pub use workspace::handle_track;

#[derive(Subcommand)]
pub enum Commands {
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
        purge: bool,
        #[arg(short, long)]
        id: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
        #[arg(short = 'P', long, default_value = "1")]
        page: usize,
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
        #[arg(short, long)]
        clear: bool,
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
