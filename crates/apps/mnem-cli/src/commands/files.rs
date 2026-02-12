use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{client::DaemonClient, protocol::methods, Repository};
use std::path::PathBuf;

use crate::ui::{self, ButlerLayout};
use crate::utils;

fn resolve_path(arg: &str) -> Result<String> {
    let mut path = PathBuf::from(arg);
    if path.is_relative() {
        path = std::env::current_dir()?.join(path);
    }
    Ok(path.to_string_lossy().to_string())
}

fn get_filename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

pub fn log(args: &[String]) -> Result<()> {
    // Advanced: mnem log --symbol <name>
    if let Some(pos) = args.iter().position(|a| a == "--symbol" || a == "-s") {
        if let Some(symbol_name) = args.get(pos + 1) {
            return log_symbol(symbol_name, args);
        }
    }

    if args.len() < 3 {
        println!("Usage: mnem log <file_path> or mnem log --symbol <name>");
        return Ok(());
    }
    let file_path = resolve_path(&args[2])?;

    let common_args = utils::parse_common_args(args);

    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::SNAPSHOT_LIST,
        serde_json::json!({
            "file_path": file_path,
            "branch": common_args.branch
        }),
    )?;

    let history: Vec<mnem_core::protocol::SnapshotInfo> = serde_json::from_value(res)?;

    // Apply pagination: N items PER branch
    let paginated_history =
        utils::paginate_per_branch(history, common_args.limit, |s| s.git_branch.clone());

    let filename = get_filename(&file_path);

    ButlerLayout::header("FILE HISTORY");
    ButlerLayout::section_start("fi", &filename);
    ButlerLayout::legend(&[
        ("●", "Latest"),
        ("●", "Past"),
        ("🏷️ ", "Commit"),
        ("🔒", "Snapshot"),
        ("🔖", "Checkpoint"),
    ]);

    let mut last_branch: Option<String> = None;

    // Fetch and display checkpoints that include this file
    if let Ok(repo) = mnem_core::Repository::init() {
        if let Ok(checkpoints) = repo.list_checkpoints() {
            let mut matching_checkpoints = Vec::new();

            for (hash, timestamp, description) in checkpoints {
                if let Ok(Some((_, file_states_json))) = repo.db.get_checkpoint_by_hash(&hash) {
                    if let Ok(file_states) =
                        serde_json::from_str::<Vec<(String, String)>>(&file_states_json)
                    {
                        if file_states.iter().any(|(path, _)| *path == file_path) {
                            matching_checkpoints.push((hash, timestamp, description));
                        }
                    }
                }
            }

            if !matching_checkpoints.is_empty() {
                ButlerLayout::section_start("🔖", "Checkpoints");
                for (hash, timestamp, description) in matching_checkpoints.iter().take(3) {
                    let hash_short = &hash[..8.min(hash.len())];
                    let timestamp_formatted = chrono::DateTime::parse_from_rfc3339(timestamp)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|_| timestamp[..16].to_string());

                    let desc = description.as_deref().unwrap_or("No description");
                    let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
                    let clickable_hash =
                        ui::Hyperlink::action(&styled_hash, "checkpoint-info", hash);

                    let content = format!("{}   {}", timestamp_formatted.dark_grey(), desc.white());
                    ButlerLayout::row_snapshot(&clickable_hash, &content);
                }
                if matching_checkpoints.len() > 3 {
                    ButlerLayout::item_simple(
                        &format!(
                            "... and {} more. Use 'mnem checkpoint-info <hash>' for details.",
                            matching_checkpoints.len() - 3
                        )
                        .dark_grey()
                        .to_string(),
                    );
                }
                ButlerLayout::section_end();
            }
        }
    }

    for (idx, s) in paginated_history.iter().enumerate() {
        let branch = s.git_branch.as_deref().unwrap_or("main");

        if last_branch.as_ref().map(|s| s.as_str()) != Some(branch) {
            if last_branch.is_some() {
                ButlerLayout::section_end();
            }
            let tag = if branch.len() >= 2 {
                &branch[..2]
            } else {
                branch
            };
            ButlerLayout::section_start(tag, branch);
            last_branch = Some(branch.to_string());
        }

        let hash_short = &s.content_hash[..7];
        let timestamp = chrono::DateTime::parse_from_rfc3339(&s.timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|_| s.timestamp[..19].replace('T', " "));

        let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
        let interactive_link = ui::Hyperlink::action(&styled_hash, "open", &s.content_hash);

        let latest_tag = if idx == 0 {
            " (latest)".green().bold()
        } else {
            "".stylize()
        };

        let content = format!("{}   {}", timestamp.dark_grey(), latest_tag);

        if idx == 0 {
            let styled_latest = hash_short.with(ui::ACTIVE).bold().to_string();
            let clickable_latest = ui::Hyperlink::action(&styled_latest, "open", &s.content_hash);
            ButlerLayout::row_snapshot_latest(&clickable_latest, &content);
        } else {
            ButlerLayout::row_snapshot(&interactive_link, &content);
        }

        if let Some(ref commit_hash) = s.commit_hash {
            let msg = s.commit_message.as_deref().unwrap_or("No message");
            println!(
                "┊      {} {}: {} - {}",
                "└──".dark_grey(),
                "🏷️  Commit".yellow(),
                commit_hash[..7].cyan(),
                msg.italic().white().dim()
            );
        }
    }

    if last_branch.is_some() {
        ButlerLayout::section_end();
    }

    if paginated_history.len() >= common_args.limit {
        ButlerLayout::item_simple(&format!(
            "... showing first {}. Use --limit <n> to see more.",
            paginated_history.len()
        ));
    }
    ButlerLayout::footer("Shift+Click the hash or 'mnem open' to view in your IDE.");
    Ok(())
}

fn log_symbol(symbol_name: &str, args: &[String]) -> Result<()> {
    let common_args = utils::parse_common_args(args);

    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::SYMBOL_GET_HISTORY,
        serde_json::json!({
            "symbol_name": symbol_name,
            "branch": common_args.branch
        }),
    )?;

    let mut response: mnem_core::protocol::SymbolHistoryResponse = serde_json::from_value(res)?;

    // Sort oldest first to calculate the evolution correctly
    response
        .history
        .sort_by(|a, b| a.snapshot.timestamp.cmp(&b.snapshot.timestamp));

    let mut processed_history = Vec::new();
    let mut last_struct_hash: Option<String> = None;
    let mut last_name: Option<String> = None;

    for entry in response.history {
        let mut status = Vec::new();

        if let Some(ln) = &last_name {
            if ln != &entry.symbol_name {
                status.push(format!("RENAMED from {}", ln.clone().yellow()));
            }
        }

        if let Some(lsh) = &last_struct_hash {
            if lsh != &entry.structural_hash {
                status.push("MODIFIED logic".blue().to_string());
            } else {
                status.push("AESTHETIC ONLY".dark_grey().to_string());
            }
        } else {
            status.push("CREATED".green().bold().to_string());
        }

        last_name = Some(entry.symbol_name.clone());
        last_struct_hash = Some(entry.structural_hash.clone());
        processed_history.push((entry, status));
    }

    processed_history.reverse();

    // Apply pagination: N items PER branch
    let paginated =
        utils::paginate_per_branch(processed_history, common_args.limit, |(entry, _)| {
            entry.snapshot.git_branch.clone()
        });

    ButlerLayout::header("SEMANTIC EVOLUTION");
    ButlerLayout::section_start("sy", &format!("Symbol: {}", symbol_name.bold().white()));
    ButlerLayout::legend(&[
        ("CREATED", "Birth"),
        ("MODIFIED", "Change"),
        ("RENAMED", "Move"),
    ]);

    let mut last_branch: Option<String> = None;
    for (entry, status) in &paginated {
        let branch = entry.snapshot.git_branch.as_deref().unwrap_or("main");

        if last_branch.as_ref().map(|s| s.as_str()) != Some(branch) {
            if last_branch.is_some() {
                ButlerLayout::section_end();
            }
            let tag = if branch.len() >= 2 {
                &branch[..2]
            } else {
                branch
            };
            ButlerLayout::section_start(tag, branch);
            last_branch = Some(branch.to_string());
        }

        let hash_short = &entry.snapshot.content_hash[..7];
        let timestamp = chrono::DateTime::parse_from_rfc3339(&entry.snapshot.timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|_| entry.snapshot.timestamp[..19].replace('T', " "));

        // --- INTERACTIVE LINK ---
        let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
        let clickable_link =
            ui::Hyperlink::action(&styled_hash, "open", &entry.snapshot.content_hash);

        let line_info = format!(
            "(L{:0>3} - L{:0>3})",
            entry.start_line + 1,
            entry.end_line + 1
        )
        .dark_grey();

        let status_styled = status
            .iter()
            .map(|s| {
                if s.contains("CREATED") {
                    s.clone().green().to_string()
                } else if s.contains("MODIFIED") {
                    s.clone().blue().to_string()
                } else if s.contains("AESTHETIC") {
                    s.clone().dark_grey().to_string()
                } else {
                    s.clone().yellow().to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        let content = format!(
            "{}  {}  {}  {}",
            timestamp.dark_grey(),
            clickable_link,
            line_info,
            status_styled
        );

        ButlerLayout::item_simple(&content);
    }

    if last_branch.is_some() {
        ButlerLayout::section_end();
    }
    if paginated.len() >= common_args.limit {
        ButlerLayout::item_simple(&format!(
            "... showing first {}. Use --limit <n> to see more.",
            paginated.len()
        ));
    }
    ButlerLayout::footer("Shift+Click the hash to open this symbol version in your IDE.");
    Ok(())
}

pub fn timeline(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        println!("Usage: mnem timeline <symbol_name>");
        return Ok(());
    }
    let symbol_name = &args[2];

    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::SYMBOL_GET_SEMANTIC_HISTORY,
        serde_json::json!({ "symbol_name": symbol_name }),
    )?;

    let resp: mnem_core::protocol::SemanticHistoryResponse = serde_json::from_value(res)?;

    ButlerLayout::header("SEMANTIC TIMELINE");
    ButlerLayout::section_start("sm", symbol_name);

    if resp.deltas.is_empty() {
        ButlerLayout::item_simple("No semantic history found for this symbol.");
    } else {
        for delta in resp.deltas {
            let kind_styled = match delta.kind {
                mnem_core::models::DeltaKind::Added => "CREATED".green().bold(),
                mnem_core::models::DeltaKind::Modified => "MODIFIED".blue().bold(),
                mnem_core::models::DeltaKind::Deleted => "DELETED".red().bold(),
                mnem_core::models::DeltaKind::Renamed => "RENAMED".yellow().bold(),
            };

            let name_info = if let Some(new_name) = delta.new_name {
                format!("{} → {}", delta.symbol_name, new_name)
            } else {
                delta.symbol_name.clone()
            };

            let hash_short = &delta.structural_hash[..8];
            let content = format!(
                "{: <10}  {: <20}  {}",
                kind_styled,
                name_info.white(),
                hash_short.dark_grey()
            );
            ButlerLayout::item_simple(&content);
        }
    }

    ButlerLayout::section_end();
    Ok(())
}

pub fn diff(args: &[String]) -> Result<()> {
    if args.len() < 4 {
        println!("Usage: mnem diff <file_path> <hash1> [hash2]");
        return Ok(());
    }
    let file_path = resolve_path(&args[2])?;

    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::SNAPSHOT_LIST,
        serde_json::json!({ "file_path": file_path }),
    )?;

    let history: Vec<mnem_core::protocol::SnapshotInfo> = serde_json::from_value(res)?;
    if history.len() < 1 {
        println!("No history found.");
        return Ok(());
    }

    let hash1 = &args[3];
    let (hash2, is_disk) = if args.len() > 4 {
        (args[4].clone(), false)
    } else {
        ("__DISK__".to_string(), true)
    };

    let res = client.call(
        methods::FILE_GET_DIFF,
        serde_json::json!({
            "file_path": file_path,
            "base_hash": Some(hash1),
            "target_hash": hash2,
        }),
    )?;

    let diff_res: mnem_core::protocol::FileDiffResponse = serde_json::from_value(res)?;
    let filename = get_filename(&file_path);

    ButlerLayout::header("FILE COMPARISON");
    ButlerLayout::section_start("di", &filename);
    let base_name = if is_disk {
        "Current Disk".yellow()
    } else {
        hash2[..8].blue().bold()
    };
    ButlerLayout::item_simple(&format!(
        "Comparing: {} -> {}",
        hash1[..8].blue().bold(),
        base_name
    ));
    println!("┊");

    for line in diff_res.diff.lines() {
        let styled_line = if line.starts_with('+') {
            line.green()
        } else if line.starts_with('-') {
            line.red()
        } else if line.starts_with("@@") {
            line.cyan().dim()
        } else {
            line.stylize()
        };
        ButlerLayout::item_simple(&styled_line.to_string());
    }
    ButlerLayout::section_end();
    Ok(())
}

pub fn info(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        println!("Usage: mnem info <file_path>");
        return Ok(());
    }
    let file_path = resolve_path(&args[2])?;

    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::FILE_GET_INFO,
        serde_json::json!({ "file_path": file_path }),
    )?;

    let info: serde_json::Value = serde_json::from_value(res)?;
    let filename = get_filename(&file_path);

    // Fetch checkpoint count for this file
    let checkpoint_count = if let Ok(repo) = Repository::init() {
        if let Ok(checkpoints) = repo.list_checkpoints() {
            checkpoints
                .iter()
                .filter(|(hash, _, _)| {
                    if let Ok(Some((_, file_states_json))) = repo.db.get_checkpoint_by_hash(hash) {
                        if let Ok(file_states) =
                            serde_json::from_str::<Vec<(String, String)>>(&file_states_json)
                        {
                            return file_states.iter().any(|(path, _)| *path == file_path);
                        }
                    }
                    false
                })
                .count()
        } else {
            0
        }
    } else {
        0
    };

    ButlerLayout::header("FILE INTELLIGENCE");
    ButlerLayout::section_start("st", &filename);

    let stats = [
        ("Path", info["path"].as_str().unwrap_or(&file_path)),
        ("Snapshots", &info["snapshot_count"].to_string()),
        ("Checkpoints", &checkpoint_count.to_string()),
        ("Size", &info["total_size_human"].as_str().unwrap_or("0 B")),
        ("Earliest", info["earliest"].as_str().unwrap_or("-")),
        ("Latest", info["latest"].as_str().unwrap_or("-")),
    ];

    for (idx, val) in stats {
        let content = format!("{: <12} {}", idx.white().dim(), val.white().bold());
        ButlerLayout::row_list("•", &content);
    }
    ButlerLayout::section_end();
    Ok(())
}

pub fn open(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        println!("Usage: mnem open <hash>");
        return Ok(());
    }
    let hash = &args[2];

    // Try to resolve the repository and original file path to get the correct extension
    let repo = Repository::find_by_hash(hash)?;
    let snapshot_info = repo
        .db
        .get_history_by_hash(hash)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve snapshot info for hash {}", hash))?;

    let extension = std::path::Path::new(&snapshot_info.file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("txt");

    // Ensure daemon is running so we can talk to it
    let _ = mnem_core::client::ensure_daemon();
    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::SNAPSHOT_GET,
        serde_json::json!({ "content_hash": hash }),
    )?;
    let content_res: serde_json::Value = serde_json::from_value(res)?;
    let content = content_res["content"].as_str().unwrap_or("");

    let hash_prefix = if hash.len() > 8 { &hash[..8] } else { hash };
    let tmp_path = format!("/tmp/mnem_snap_{}.{}", hash_prefix, extension);
    std::fs::write(&tmp_path, content)?;

    let ide = {
        let config_manager = repo
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        config_manager.config.ide
    };

    ButlerLayout::header("OPENING SNAPSHOT");
    ButlerLayout::item_simple(&format!(
        "{} Target IDE: {}",
        "→".cyan(),
        ide.as_str().bold().white()
    ));
    ButlerLayout::item_simple(&format!(
        "{} Hash: {}",
        "→".cyan(),
        hash.as_str().with(ui::ACCENT).bold()
    ));

    let mut cmd = std::process::Command::new("open");
    match ide {
        mnem_core::config::Ide::Zed => {
            cmd.arg("-a").arg("Zed");
        }
        mnem_core::config::Ide::VsCode => {
            cmd.arg("-a").arg("Visual Studio Code");
        }
        mnem_core::config::Ide::ZedPreview => {
            cmd.arg("-a").arg("Zed Preview");
        }
    }
    cmd.arg(&tmp_path);

    if let Err(e) = cmd.spawn() {
        eprintln!("{} Failed to launch IDE: {}", "✘".red(), e);
    }

    Ok(())
}

pub fn cat(args: &[String]) -> Result<()> {
    if args.len() < 3 {
        println!("Usage: mnem cat <hash>");
        return Ok(());
    }
    let hash = &args[2];

    let _ = mnem_core::client::ensure_daemon();

    let mut client = DaemonClient::connect()?;
    let res = client.call(
        methods::SNAPSHOT_GET,
        serde_json::json!({ "content_hash": hash }),
    )?;
    let content_res: serde_json::Value = serde_json::from_value(res)?;
    let content = content_res["content"].as_str().unwrap_or("");

    println!("{}", content);
    Ok(())
}

pub fn restore(args: &[String]) -> Result<()> {
    // Check for --checkpoint flag
    if let Some(pos) = args.iter().position(|a| a == "--checkpoint" || a == "-c") {
        if let Some(hash) = args.get(pos + 1) {
            let repo = Repository::init()?;
            let count = repo.revert_to_checkpoint(hash)?;

            ButlerLayout::header("PROJECT RESTORED");
            ButlerLayout::section_start("rs", "Massive Rollback");
            ButlerLayout::item_simple(&format!(
                "{} Reverted to checkpoint: {}",
                "✓".green(),
                hash.as_str().cyan().bold()
            ));
            ButlerLayout::item_simple(&format!(
                "{} Restored {} files.",
                "→".cyan(),
                count.to_string().white().bold()
            ));
            ButlerLayout::section_end();
            return Ok(());
        }
    }

    if args.len() < 4 {
        println!("Usage: mnem restore <file_path> <content_hash> [--symbol <name>]");
        println!("       mnem restore --checkpoint <hash>");
        return Ok(());
    }
    let file_path = resolve_path(&args[2])?;
    let hash = &args[3];

    // Check for --symbol flag
    let symbol_name = args
        .iter()
        .position(|a| a == "--symbol" || a == "-s")
        .and_then(|i| args.get(i + 1));

    let mut client = DaemonClient::connect()?;

    if let Some(symbol) = symbol_name {
        let _ = client.call(
            methods::SNAPSHOT_RESTORE_SYMBOL_V1,
            serde_json::json!({
                "content_hash": hash,
                "target_path": file_path,
                "symbol_name": symbol,
            }),
        )?;

        ButlerLayout::header("SURGICAL RESTORE");
        ButlerLayout::section_start("rs", "Symbol Transplant");
        ButlerLayout::item_simple(&format!(
            "{} Restored symbol '{}' from snapshot {}",
            "✓".green(),
            symbol.clone().bold().white(),
            hash[..8].cyan()
        ));
    } else {
        let _ = client.call(
            methods::SNAPSHOT_RESTORE_V1,
            serde_json::json!({
                "content_hash": hash,
                "target_path": file_path,
            }),
        )?;

        ButlerLayout::header("RESTORE COMPLETED");
        ButlerLayout::section_start("rs", "FileSystem Sync");
        ButlerLayout::item_simple(&format!(
            "{} File restored: {} to {}",
            "✓".green(),
            hash.clone().bold().cyan(),
            file_path.white()
        ));
    }
    ButlerLayout::section_end();
    Ok(())
}

pub fn search(args: &[String]) -> Result<()> {
    let mut query = String::new();
    let mut limit = 50;
    let mut file_filter = None;
    let mut symbol_mode = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--limit" | "-n" => {
                if let Some(val) = args.get(i + 1) {
                    if let Ok(l) = val.parse() {
                        limit = l;
                    }
                    i += 1;
                }
            }
            "--file" | "-f" => {
                if let Some(val) = args.get(i + 1) {
                    file_filter = Some(val.clone());
                    i += 1;
                }
            }
            "--symbol" | "-s" => {
                symbol_mode = true;
            }
            arg if !arg.starts_with('-') && query.is_empty() => {
                query = arg.to_string();
            }
            _ => {}
        }
        i += 1;
    }

    if query.is_empty() {
        println!("Usage: mnem search <query> [options]");
        println!("\nOptions:");
        println!("  -n, --limit <n>    Maximum number of results (default: 50)");
        println!("  -f, --file <path>  Filter by file path");
        println!("  -s, --symbol       Search for symbols instead of raw text");
        println!("\nExamples:");
        println!("  mnem search \"main\" --file main.rs");
        println!("  mnem search \"UserRepository\" --symbol");
        return Ok(());
    }

    let _ = mnem_core::client::ensure_daemon();
    let mut client = DaemonClient::connect()?;

    if symbol_mode {
        // Implement Symbol Search
        let res = client.call(
            methods::SYMBOL_SEARCH,
            serde_json::json!({ "query": query }),
        )?;

        let mut locations: Vec<mnem_core::protocol::SymbolLocation> = serde_json::from_value(res)?;

        // Apply file filter if present
        if let Some(ref filter) = file_filter {
            locations.retain(|l| l.file_path.contains(filter));
        }

        locations.truncate(limit);

        if locations.is_empty() {
            ButlerLayout::header("SYMBOL SEARCH");
            ButlerLayout::item_simple(&format!(
                "{}  No symbols found matching \"{}\"",
                "!".yellow(),
                query.bold().white()
            ));
            return Ok(());
        }

        ButlerLayout::header("SYMBOL SEARCH");
        let mut grouped_by_file: std::collections::HashMap<
            String,
            Vec<mnem_core::protocol::SymbolLocation>,
        > = std::collections::HashMap::new();
        let mut file_order = Vec::new();

        for loc in locations {
            if !grouped_by_file.contains_key(&loc.file_path) {
                file_order.push(loc.file_path.clone());
            }
            grouped_by_file
                .entry(loc.file_path.clone())
                .or_default()
                .push(loc);
        }

        for path in file_order {
            let filename = get_filename(&path);
            // SAFETY: file_order is built from grouped_by_file keys, so this will always succeed
            let locs = grouped_by_file
                .get(&path)
                .expect("file_order contains only valid keys from grouped_by_file");
            ButlerLayout::section_start("sy", &filename);
            ButlerLayout::item_simple(&path.dark_grey().to_string());

            for loc in locs {
                println!(
                    "┊   {} {} {} [{}-{}]",
                    "•".cyan(),
                    loc.kind.as_str().blue().bold(),
                    loc.name.as_str().bold().white(),
                    loc.start_line,
                    loc.end_line
                );
            }
            ButlerLayout::section_end();
        }
        ButlerLayout::footer("Use 'mnem log --symbol <name>' to see version history of a symbol.");
        return Ok(());
    }

    // Default: Content Search
    let res = client.call(
        methods::CONTENT_SEARCH_V1,
        serde_json::json!({
            "query": query,
            "limit": limit,
            "path_filter": file_filter
        }),
    )?;

    let results: Vec<mnem_core::models::SearchResult> =
        serde_json::from_value(res["results"].clone()).unwrap_or_default();

    if results.is_empty() {
        ButlerLayout::header("SEARCH RESULTS");
        ButlerLayout::item_simple(&format!(
            "{}  No results found for \"{}\"{}",
            "!".yellow(),
            query.bold().white(),
            file_filter
                .as_ref()
                .map(|f| format!(" in files matching \"{}\"", f))
                .unwrap_or_default()
        ));
        ButlerLayout::footer("");
        return Ok(());
    }

    ButlerLayout::header("SEARCH RESULTS");

    // Get unique file paths in order of appearance
    let mut files = Vec::new();
    for r in &results {
        if !files.contains(&r.file_path) {
            files.push(r.file_path.clone());
        }
    }

    for file_path in files {
        let filename = get_filename(&file_path);
        let matches_in_file: Vec<&mnem_core::models::SearchResult> = results
            .iter()
            .filter(|r| r.file_path == file_path)
            .collect();

        ButlerLayout::section_start("fi", &filename);
        ButlerLayout::item_simple(&file_path.dark_grey().to_string());

        // Group by content_hash (snapshot)
        let mut hashes = Vec::new();
        for r in &matches_in_file {
            if !hashes.contains(&r.content_hash) {
                hashes.push(r.content_hash.clone());
            }
        }

        for hash in hashes {
            let matches_in_snap: Vec<&&mnem_core::models::SearchResult> = matches_in_file
                .iter()
                .filter(|r| r.content_hash == hash)
                .collect();

            let first = matches_in_snap[0];
            let hash_short = &hash[..7];
            let branch = first.git_branch.as_deref().unwrap_or("?");
            let timestamp = chrono::DateTime::parse_from_rfc3339(&first.timestamp)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| first.timestamp[..16].replace('T', " "));

            let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
            let clickable_link = ui::Hyperlink::action(&styled_hash, "open", &hash);

            let meta = format!(
                "{}   {}   {}",
                timestamp.dark_grey(),
                branch.cyan().italic(),
                format!("({} matches)", matches_in_snap.len()).dim()
            );

            ButlerLayout::row_snapshot(&clickable_link, &meta);

            for m in matches_in_snap {
                let highlighted = highlight_match(&m.content, &query);
                println!(
                    "┊{: >12} {}",
                    format!("L{}", m.line_number).dark_grey(),
                    highlighted
                );
            }
        }
        ButlerLayout::section_end();
    }

    if results.len() >= limit {
        ButlerLayout::item_simple(&format!(
            "... showing first {}. Use --limit <n> to see more.",
            limit
        ));
    }
    ButlerLayout::footer("Shift+Click the hash to open the snapshot version in your IDE.");
    Ok(())
}

/// Highlights matching substring in a line by wrapping it in yellow.
fn highlight_match(line: &str, query: &str) -> String {
    if let Some(idx) = line.to_lowercase().find(&query.to_lowercase()) {
        let before = &line[..idx];
        let matched = &line[idx..idx + query.len()];
        let after = &line[idx + query.len()..];
        format!(
            "{}{}{}",
            before.white(),
            matched.yellow().bold(),
            after.white()
        )
    } else {
        line.white().to_string()
    }
}
