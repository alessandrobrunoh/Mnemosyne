use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Renderable};
use mnem_core::models::SearchResult;
use mnem_core::protocol::SymbolLocation;

#[derive(Serialize)]
pub struct SearchResponse {
    pub success: bool,
    pub query: String,
    pub semantic: bool,
    pub results: SearchResults,
    pub limit: usize,
    pub page: usize,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SearchResults {
    Semantic(Vec<SymbolLocation>),
    Content(Vec<SearchResult>),
}

impl Renderable for SearchResponse {
    fn text(&self) -> Result<()> {
        use crossterm::style::Stylize;
        let layout = Layout::new();

        match &self.results {
            SearchResults::Semantic(locations) => {
                layout.header("SYMBOL SEARCH");

                if locations.is_empty() {
                    layout.item_simple(&format!(
                        "{}  No symbols found matching \"{}\"",
                        "!".yellow(),
                        self.query.clone().bold().white()
                    ));
                    return Ok(());
                }

                let mut grouped: std::collections::HashMap<String, Vec<SymbolLocation>> =
                    std::collections::HashMap::new();
                let mut file_order: Vec<String> = Vec::new();

                for loc in locations {
                    if !grouped.contains_key(&loc.file_path) {
                        file_order.push(loc.file_path.clone());
                    }
                    grouped
                        .entry(loc.file_path.clone())
                        .or_default()
                        .push(loc.clone());
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
            }
            SearchResults::Content(results) => {
                layout.header("SEARCH RESULTS");

                if results.is_empty() {
                    layout.item_simple(&format!(
                        "{}  No results found for \"{}\"",
                        "!".yellow(),
                        self.query.clone().bold().white()
                    ));
                    return Ok(());
                }

                layout.empty();

                let mut files: Vec<String> = Vec::new();
                for r in results {
                    if !files.contains(&r.file_path) {
                        files.push(r.file_path.clone());
                    }
                }

                for file_path in &files {
                    let filename = std::path::Path::new(file_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| file_path.clone());

                    let matches_in_file: Vec<&SearchResult> = results
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
                        let matches_in_snap: Vec<&&SearchResult> = matches_in_file
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

                        let styled_hash = hash_short
                            .with(crate::ui::colors::CYAN_BRIGHT)
                            .bold()
                            .to_string();
                        let clickable_link =
                            crate::ui::Hyperlink::action(&styled_hash, "open", hash);

                        let meta = format!(
                            "{}  {}  [{}]",
                            timestamp.with(crossterm::style::Color::DarkGrey),
                            branch.cyan().italic(),
                            clickable_link
                        );
                        layout.row_labeled("◆", &meta, &matches_in_snap.len().to_string());
                        layout.empty();

                        for r in matches_in_snap {
                            let line_num = r.line_number;
                            let line_content = &r.content;

                            let highlighted = if line_content.is_empty() {
                                "•".dark_grey().to_string()
                            } else {
                                highlight_match(line_content, &self.query)
                            };

                            println!(
                                "  {}  {}  {}",
                                line_num.to_string().dark_grey(),
                                highlighted,
                                r.git_branch
                                    .as_deref()
                                    .map(|b| b.cyan().to_string())
                                    .unwrap_or_default()
                            );
                        }
                        layout.empty();
                    }
                }
            }
        }
        Ok(())
    }
}

/// Search through code history
#[derive(Args, Clone, Debug)]
pub struct SearchCommand {
    /// Search query
    query: Option<String>,

    /// Filter by file path
    #[arg(short, long)]
    file: Option<String>,

    /// Maximum number of results
    #[arg(short, long, default_value = "20")]
    limit: usize,

    /// Page number
    #[arg(short = 'P', long, default_value = "1")]
    page: usize,

    /// Search for symbols instead of raw text
    #[arg(short, long)]
    semantic: bool,
}

impl CommandStrategy for SearchCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        use mnem_core::{client::DaemonClient, protocol::methods};

        let Some(query) = self.query.as_ref().filter(|q| !q.is_empty()) else {
            if global_opts.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": "Missing query",
                        "code": "MISSING_QUERY"
                    })
                );
            } else {
                let layout = Layout::new();
                layout.usage(
                    "s",
                    "<query> [--file <path>] [--limit <n>] [--page <p>] [--semantic]",
                );
                layout.empty();
                layout.item_simple("Options:");
                layout.row_list("-f, --file <path>", "Filter by file path");
                layout.row_list("-n, --limit <n>", "Maximum number of results (default: 20)");
                layout.row_list("-P, --page <p>", "Page number (default: 1)");
                layout.row_list("-s, --semantic", "Search for symbols instead of raw text");
                layout.empty();
                layout.item_simple("Examples:");
                layout.item_simple("  mnem s \"main\" --file main.rs");
                layout.item_simple("  mnem s \"UserRepository\" --semantic");
            }
            return Ok(());
        };

        let offset = (self.page.saturating_sub(1)) * self.limit;

        let mut client = match DaemonClient::connect() {
            Ok(c) => c,
            Err(_) => {
                if global_opts.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": "Daemon not running",
                            "code": "DAEMON_NOT_RUNNING"
                        })
                    );
                } else {
                    Layout::new().error("Daemon is not running. Start it with 'mnem on'");
                }
                return Ok(());
            }
        };

        let results = if self.semantic {
            let res = client.call(
                methods::SYMBOL_SEARCH,
                serde_json::json!({ "query": query }),
            )?;

            let mut locations: Vec<SymbolLocation> = serde_json::from_value(res)?;

            if let Some(ref filter) = self.file {
                locations.retain(|l| l.file_path.contains(filter.as_str()));
            }

            // Apply manual pagination for symbols for now
            let paginated: Vec<SymbolLocation> = locations
                .into_iter()
                .skip(offset)
                .take(self.limit)
                .collect();
            SearchResults::Semantic(paginated)
        } else {
            let res = client.call(
                methods::CONTENT_SEARCH_V1,
                serde_json::json!({
                    "query": query,
                    "limit": self.limit,
                    "offset": offset,
                    "path_filter": self.file
                }),
            )?;

            let results: Vec<SearchResult> =
                serde_json::from_value(res["results"].clone()).unwrap_or_default();
            SearchResults::Content(results)
        };

        let response = SearchResponse {
            success: true,
            query: query.to_string(),
            semantic: self.semantic,
            results,
            limit: self.limit,
            page: self.page,
        };

        if global_opts.json {
            println!("{}", response.json()?);
        } else {
            response.text()?;
        }

        Ok(())
    }
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
