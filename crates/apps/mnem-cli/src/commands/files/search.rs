use crate::commands::Command;
use crate::ui::{self, Layout};
use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::{client::DaemonClient, protocol::methods};

fn get_filename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

#[derive(Debug)]
pub struct SearchCommand;

impl Command for SearchCommand {
    fn name(&self) -> &str {
        "search"
    }

    fn usage(&self) -> &str {
        "<query> [options]"
    }

    fn description(&self) -> &str {
        "Search for text or symbols across snapshots"
    }

    fn group(&self) -> &str {
        "Files"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["grep"]
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let layout = Layout::new();
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
            layout.usage(self.name(), self.usage());
            layout.empty();
            layout.item_simple("Options:");
            layout.row_list("-n, --limit <n>", "Maximum number of results (default: 50)");
            layout.row_list("-f, --file <path>", "Filter by file path");
            layout.row_list("-s, --symbol", "Search for symbols instead of raw text");
            layout.empty();
            layout.item_simple("Examples:");
            layout.item_simple("  mnem search \"main\" --file main.rs");
            layout.item_simple("  mnem search \"UserRepository\" --symbol");
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

            let mut locations: Vec<mnem_core::protocol::SymbolLocation> =
                serde_json::from_value(res)?;

            // Apply file filter if present
            if let Some(ref filter) = file_filter {
                locations.retain(|l| l.file_path.contains(filter));
            }

            locations.truncate(limit);

            if locations.is_empty() {
                layout.header("SYMBOL SEARCH");
                layout.item_simple(&format!(
                    "{}  No symbols found matching \"{}\"",
                    "!".yellow(),
                    query.bold().white()
                ));
                return Ok(());
            }

            layout.header("SYMBOL SEARCH");
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
                let locs = grouped_by_file.get(&path).unwrap();
                layout.section_start("sy", &filename);
                layout.item_simple(&path.dark_grey().to_string());

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
                layout.section_end();
            }
            layout.footer("Use 'mnem log --symbol <name>' to see version history of a symbol.");
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
            layout.header("SEARCH RESULTS");
            layout.item_simple(&format!(
                "{}  No results found for \"{}\"{}",
                "!".yellow(),
                query.bold().white(),
                file_filter
                    .as_ref()
                    .map(|f| format!(" in files matching \"{}\"", f))
                    .unwrap_or_default()
            ));
            return Ok(());
        }

        layout.header("SEARCH RESULTS");

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

            layout.section_start("fi", &filename);
            layout.item_simple(&file_path.dark_grey().to_string());

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

                layout.row_snapshot(&clickable_link, &meta);

                for m in matches_in_snap {
                    let highlighted = highlight_match(&m.content, &query);
                    println!(
                        "┊{: >12} {}",
                        format!("L{}", m.line_number).dark_grey(),
                        highlighted
                    );
                }
            }
            layout.section_end();
        }

        if results.len() >= limit {
            layout.item_simple(&format!(
                "... showing first {}. Use --limit <n> to see more.",
                limit
            ));
        }
        layout.footer("Shift+Click the hash to open the snapshot version in your IDE.");
        Ok(())
    }
}

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
