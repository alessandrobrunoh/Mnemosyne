use crate::i18n;
use crate::ui::ButlerLayout;
use anyhow::Result;
use crossterm::style::Stylize;

pub fn help() -> Result<()> {
    let msg = i18n::current();

    ButlerLayout::section_start(
        "mn",
        &format!("{} [v{}]", msg.app_name(), env!("CARGO_PKG_VERSION")),
    );
    ButlerLayout::item_simple(&msg.tagline().italic().dark_grey().to_string());

    ButlerLayout::item_simple("");
    ButlerLayout::item_simple(&msg.core_ops_header().bold().cyan().to_string());
    let core = [
        ("(default)", "Show project activity (if in repo) or list"),
        ("tui", msg.cmd_default_desc()),
        ("start", msg.cmd_start_desc()),
        ("stop", msg.cmd_stop_desc()),
        ("version", "Show version info and ASCII banner"),
    ];
    for (cmd, desc) in core {
        ButlerLayout::row_list(cmd.green().to_string().as_str(), desc);
    }

    ButlerLayout::item_simple("");
    ButlerLayout::item_simple(&msg.project_history_header().bold().cyan().to_string());
    let history = [
        ("list", msg.cmd_list_desc()),
        (
            "watch [-p]",
            "Track current or specific project in background",
        ),
        (
            "forget <id>",
            "Remove a project from registry (use --prune to nuke data)",
        ),
        (
            "project <id>",
            "Show detailed info and stats for a specific project",
        ),
        ("log <file>", msg.cmd_log_desc()),
        ("timeline <s>", "Show semantic evolution of a symbol"),
        ("search <q>", msg.cmd_search_desc()),
        ("diff <f> <h>", "Show differences between file versions"),
        ("history", "Show project or global history feed"),
        ("statistics", "Show cool and interesting project metrics"),
        (
            "open <hash>",
            "Open a snapshot in your IDE (Zed, VSCode...)",
        ),
        ("restore <f>", "Restore a file to a specific hash"),
        (
            "checkpoint",
            "Create a manual project-wide semantic snapshot",
        ),
        ("commits", "List all Git commits"),
        ("commit-info <hash>", "Show details of a specific commit"),
        ("log-commits", "Show commits with modified files"),
    ];
    for (cmd, desc) in history {
        ButlerLayout::row_list(cmd.green().to_string().as_str(), desc);
    }

    ButlerLayout::item_simple("");
    ButlerLayout::item_simple(&msg.maintenance_header().bold().cyan().to_string());
    let maint = [
        ("status", msg.cmd_status_desc()),
        ("config set", msg.cmd_config_desc()),
        ("gc", "Prune old snapshots and optimize storage"),
    ];
    for (cmd, desc) in maint {
        ButlerLayout::row_list(cmd.green().to_string().as_str(), desc);
    }

    ButlerLayout::item_simple("");
    ButlerLayout::item_simple(&"INTEGRATIONS".bold().yellow().to_string());
    ButlerLayout::row_list(
        "git-hook".green().to_string().as_str(),
        "Install a post-commit hook for automatic checkpoints",
    );
    ButlerLayout::row_list(
        "git-event".green().to_string().as_str(),
        "Internal command to link Git commits (called by hook)",
    );

    ButlerLayout::section_end();
    let footer_text = format!(
        "{}: https://github.com/alessandrobrunoh/mnemosyne",
        msg.learn_more()
    );
    ButlerLayout::footer(&footer_text);

    Ok(())
}

pub fn version() -> Result<()> {
    let msg = i18n::current();
    println!("");
    println!("{}", " ███╗   ███╗███╗   ██╗███████╗███╗   ███╗".cyan());
    println!("{}", " ████╗ ████║████╗  ██║██╔════╝████╗ ████║".cyan());
    println!("{}", " ██╔████╔██║██╔██╗ ██║█████╗  ██╔████╔██║".cyan());
    println!("{}", " ██║╚██╔╝██║██║╚██╗██║██╔══╝  ██║╚██╔╝██║".cyan());
    println!("{}", " ██║ ╚═╝ ██║██║ ╚████║███████╗██║ ╚═╝ ██║".cyan());
    println!("{}", " ╚═╝     ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚═╝".cyan());

    ButlerLayout::section_start("v", &format!("mnemosyne v{}", env!("CARGO_PKG_VERSION")));
    ButlerLayout::item_simple(&msg.tagline().italic().dark_grey().to_string());
    ButlerLayout::section_end();
    Ok(())
}
