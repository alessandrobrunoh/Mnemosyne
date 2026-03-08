use anyhow::Result;

use crate::ui::Layout;

pub fn handle_s(
    query: Option<String>,
    file: Option<String>,
    limit: Option<usize>,
    semantic: bool,
) -> Result<()> {
    use crate::ui;
    use crossterm::style::Stylize;
    use mnem_core::{client::DaemonClient, protocol::methods};

    let layout = Layout::new();

    let query = match query {
        Some(q) if !q.is_empty() => q,
        _ => {
            layout.usage("s", "<query> [--file <path>] [--limit <n>] [--semantic]");
            layout.empty();
            layout.item_simple("Options:");
            layout.row_list("-f, --file <path>", "Filter by file path");
            layout.row_list("-n, --limit <n>", "Maximum number of results (default: 50)");
            layout.row_list("-s, --semantic", "Search for symbols instead of raw text");
            layout.empty();
            layout.item_simple("Examples:");
            layout.item_simple("  mnem s \"main\" --file main.rs");
            layout.item_simple("  mnem s \"UserRepository\" --semantic");
            return Ok(());
        }
    };

    let limit_val = limit.unwrap_or(50);

    let mut client = match DaemonClient::connect() {
        Ok(c) => c,
        Err(_) => {
            layout.error("Daemon is not running. Start it with 'mnem on'");
            return Ok(());
        }
    };

    if semantic {
        let res = client.call(
            methods::SYMBOL_SEARCH,
            serde_json::json!({ "query": query }),
        )?;

        let mut locations: Vec<mnem_core::protocol::SymbolLocation> = serde_json::from_value(res)?;

        if let Some(ref filter) = file {
            locations.retain(|l| l.file_path.contains(filter.as_str()));
        }

        locations.truncate(limit_val);

        layout.header("SYMBOL SEARCH");

        if locations.is_empty() {
            layout.item_simple(&format!(
                "{}  No symbols found matching \"{}\"",
                "!".yellow(),
                query.bold().white()
            ));
            return Ok(());
        }

        let mut grouped: std::collections::HashMap<
            String,
            Vec<mnem_core::protocol::SymbolLocation>,
        > = std::collections::HashMap::new();
        let mut file_order: Vec<String> = Vec::new();

        for loc in locations {
            if !grouped.contains_key(&loc.file_path) {
                file_order.push(loc.file_path.clone());
            }
            grouped.entry(loc.file_path.clone()).or_default().push(loc);
        }

        for path in file_order {
            let filename = std::path::Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());

            let locs = grouped.get(&path).unwrap();
            layout.section_start("sy", &filename);
            layout.item_simple(&path.dark_grey().to_string());

            for loc in locs {
                println!(
                    "┃   {} {} {} [{}-{}]",
                    "•".cyan(),
                    loc.kind.as_str().blue().bold(),
                    loc.name.as_str().bold().white(),
                    loc.start_line,
                    loc.end_line
                );
            }
            layout.section_end();
        }

        layout.footer("Use 'mnem h --symbol <name>' to see version history of a symbol.");
    } else {
        let res = client.call(
            methods::CONTENT_SEARCH_V1,
            serde_json::json!({
                "query": query,
                "limit": limit_val,
                "path_filter": file
            }),
        )?;

        let results: Vec<mnem_core::models::SearchResult> =
            serde_json::from_value(res["results"].clone()).unwrap_or_default();

        layout.header("SEARCH RESULTS");

        if results.is_empty() {
            layout.item_simple(&format!(
                "{}  No results found for \"{}\"{}",
                "!".yellow(),
                query.bold().white(),
                file.as_ref()
                    .map(|f| format!(" in files matching \"{}\"", f))
                    .unwrap_or_default()
            ));
            return Ok(());
        }

        layout.empty();

        let mut files: Vec<String> = Vec::new();
        for r in &results {
            if !files.contains(&r.file_path) {
                files.push(r.file_path.clone());
            }
        }

        for file_path in &files {
            let filename = std::path::Path::new(file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.clone());

            let matches_in_file: Vec<&mnem_core::models::SearchResult> = results
                .iter()
                .filter(|r| &r.file_path == file_path)
                .collect();

            layout.item_simple(&format!("{} {}", "📄".cyan(), filename.bold().white()));
            layout.item_simple(&file_path.clone().dark_grey().to_string());
            layout.empty();

            let mut hashes: Vec<String> = Vec::new();
            for r in &matches_in_file {
                if !hashes.contains(&r.content_hash) {
                    hashes.push(r.content_hash.clone());
                }
            }

            for hash in &hashes {
                let matches_in_snap: Vec<&&mnem_core::models::SearchResult> = matches_in_file
                    .iter()
                    .filter(|r| &r.content_hash == hash)
                    .collect();

                let first = matches_in_snap[0];
                let hash_short = &hash[..7.min(hash.len())];
                let branch = first.git_branch.as_deref().unwrap_or("?");
                let timestamp = chrono::DateTime::parse_from_rfc3339(&first.timestamp)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|_| {
                        first.timestamp[..16.min(first.timestamp.len())].replace('T', " ")
                    });

                let styled_hash = hash_short.with(ui::ACCENT).bold().to_string();
                let clickable_link = ui::Hyperlink::action(&styled_hash, "open", hash);

                let meta = format!(
                    "{}  {}  [{}]",
                    timestamp.with(crossterm::style::Color::DarkGrey),
                    branch.cyan().italic(),
                    format!("{} match(es)", matches_in_snap.len())
                );
                layout.row_snapshot(&clickable_link, &meta);

                for m in matches_in_snap {
                    let highlighted = highlight_match(&m.content, &query);
                    println!(
                        "┃   L{: >4}  {}",
                        m.line_number.to_string().dark_grey(),
                        highlighted
                    );
                }
            }
            layout.empty();
        }

        if results.len() >= limit_val {
            layout.item_simple(&format!(
                "... showing first {}. Use --limit <n> to see more.",
                limit_val
            ));
        }

        layout.footer_hint("Click on hash to open that version in your IDE");
    }

    Ok(())
}

fn highlight_match(line: &str, query: &str) -> String {
    use crossterm::style::Stylize;

    if let Some(idx) = line.to_lowercase().find(&query.to_lowercase()) {
        let end = (idx + query.len()).min(line.len());
        let before = &line[..idx];
        let matched = &line[idx..end];
        let after = &line[end..];
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
