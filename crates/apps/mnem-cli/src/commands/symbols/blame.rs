use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, Renderable};
use mnem_core::storage::Repository;

#[derive(Serialize)]
pub struct BlameResponse {
    pub success: bool,
    pub symbol_name: String,
    pub file_path: String,
    pub history: Vec<SymbolVersion>,
    pub stats: Option<SymbolStats>,
}

#[derive(Serialize)]
pub struct SymbolVersion {
    pub version: usize,
    pub timestamp: String,
    pub git_user: String,
    pub changes: Vec<String>,
    pub complexity: usize,
    pub lines: (usize, usize),
}

#[derive(Serialize)]
pub struct SymbolStats {
    pub total_changes: usize,
    pub contributors: Vec<ContributorStats>,
    pub high_churn: bool,
    pub bug_rate: f64,
    pub avg_complexity: usize,
}

#[derive(Serialize)]
pub struct ContributorStats {
    pub user: String,
    pub email: String,
    pub change_count: usize,
    pub percentage: f64,
}

impl Renderable for BlameResponse {
    fn text(&self) -> Result<()> {
        let layout = Layout::new();
        let theme = layout.theme();

        layout.header_dashboard(&format!("SYMBOL BLAME: {}", self.symbol_name));

        if self.history.is_empty() {
            layout.item_simple(&format!(
                "{} No history found for symbol '{}'",
                "!", self.symbol_name
            ));
            layout.empty();
            layout.info(&format!(
                "Make sure the symbol exists in: {}",
                self.file_path
            ));
            return Ok(());
        }

        // Show timeline
        layout.section_branch("sb", "Evolution Timeline");

        for (idx, version) in self.history.iter().enumerate() {
            let version_num = idx + 1;

            layout.graph_connector();
            layout.graph_block_header(
                &format!("v{}", version_num),
                &version.timestamp,
                theme.timeline_cyan,
            );

            // Git user attribution
            layout.graph_node(
                &version.git_user,
                "AUTHOR",
                true,
                "changed",
                None,
                theme.text_dim,
            );

            // Changes
            for change in &version.changes {
                layout.graph_node(change, "CHANGE", false, "", None, theme.text_dim);
            }

            // Complexity
            layout.graph_node(
                &format!("{}", version.complexity),
                "COMPLEXITY",
                false,
                "lines",
                None,
                theme.text_dim,
            );

            // Line numbers
            layout.graph_node(
                &format!("{}-{}", version.lines.0, version.lines.1),
                "LINES",
                false,
                "range",
                None,
                theme.text_dim,
            );
        }

        layout.graph_branch_end();
        layout.empty();

        // Show statistics if available
        if let Some(ref stats) = self.stats {
            layout.header_dashboard("CONTRIBUTOR STATISTICS");

            // High churn warning
            if stats.high_churn {
                layout.badge_error(
                    "HIGH CHURN",
                    &format!("{} changes - consider refactoring", stats.total_changes),
                );
            }

            // Bug rate warning
            if stats.bug_rate > 0.3 {
                layout.badge_error(
                    "BUG RATE",
                    &format!("{:.1}% - high instability", stats.bug_rate * 100.0),
                );
            } else if stats.bug_rate > 0.1 {
                layout.badge_info("BUG RATE", &format!("{:.1}%", stats.bug_rate * 100.0));
            }

            layout.empty();

            // Contributors
            layout.section_branch("cn", "Contributors");

            let total = stats.total_changes;
            for contributor in &stats.contributors {
                let percentage = (contributor.change_count as f64 / total as f64) * 100.0;

                layout.graph_connector();
                layout.graph_node(
                    &format!("{} changes", contributor.change_count),
                    &contributor.user,
                    true,
                    &format!("{:.1}%", percentage),
                    None,
                    theme.success_bright,
                );
            }

            layout.graph_branch_end();
            layout.empty();

            // Average complexity
            layout.item_simple(&format!(
                "{} Average complexity: {} lines",
                "•".cyan(),
                stats.avg_complexity
            ));
        }

        layout.footer("Use 'mnem diff' to see detailed changes between versions");

        Ok(())
    }
}

#[derive(Args, Clone, Debug)]
pub struct BlameCommand {
    /// File path containing the symbol
    pub file: String,

    /// Symbol name to blame
    pub symbol: String,

    /// Show statistics only
    #[arg(short, long)]
    stats_only: bool,

    /// Filter by git user email
    #[arg(short, long)]
    user: Option<String>,

    /// Limit number of versions
    #[arg(short, long, default_value = "50")]
    limit: usize,
}

impl CommandStrategy for BlameCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        use mnem_core::env::get_base_dir;

        let base_dir = get_base_dir()?;
        let cwd = std::env::current_dir()?;

        // Resolve project path
        let project_path = if std::path::Path::new(&self.file).is_absolute() {
            std::path::PathBuf::from(&self.file)
        } else {
            cwd.join(&self.file)
        };

        // Get parent directory as project root
        let project_root = project_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine project root"))?;

        let repo = match Repository::open(base_dir, project_root.to_path_buf()) {
            Ok(r) => r,
            Err(_) => {
                if global_opts.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": "Project not tracked",
                            "code": "PROJECT_NOT_TRACKED"
                        })
                    );
                } else {
                    Layout::new().error("Project not tracked. Run 'mnem track' first.");
                }
                return Ok(());
            }
        };

        // Get symbol history from database
        let history = match repo.db.get_symbol_history_with_users(&self.symbol) {
            Ok(h) => h,
            Err(_) => {
                if global_opts.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": "Symbol not found in history",
                            "code": "SYMBOL_NOT_FOUND"
                        })
                    );
                } else {
                    Layout::new().error(&format!("Symbol '{}' not found in history", self.symbol));
                }
                return Ok(());
            }
        };

        // Get symbol statistics
        let stats = match repo.db.get_symbol_stats(&self.symbol) {
            Ok(s) => s,
            Err(_) => {
                // If stats fail, continue without them
                if global_opts.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": "Failed to get statistics",
                            "code": "STATS_ERROR"
                        })
                    );
                    return Ok(());
                } else {
                    Layout::new().error("Failed to get statistics");
                    return Ok(());
                }
            }
        };

        // Filter by user if specified
        let filtered_history = if let Some(ref user_email) = self.user {
            history
                .into_iter()
                .filter(|h| h.git_user_email == *user_email)
                .collect()
        } else {
            history
        };

        // Apply limit
        let limited_history: Vec<_> = filtered_history.into_iter().take(self.limit).collect();

        // Convert to response format
        let versions: Vec<SymbolVersion> = limited_history
            .iter()
            .enumerate()
            .map(|(idx, h)| {
                let changes = vec![format!("{} {}", h.symbol_kind, h.symbol_name)];

                SymbolVersion {
                    version: idx + 1,
                    timestamp: h.timestamp.clone(),
                    git_user: format!("{} <{}>", h.git_user_name, h.git_user_email),
                    changes,
                    complexity: h.complexity,
                    lines: (h.start_line, h.end_line),
                }
            })
            .collect();

        let contributor_stats: Vec<ContributorStats> = stats
            .contributors
            .iter()
            .map(|c| {
                let percentage = if stats.total_changes > 0 {
                    (c.change_count as f64 / stats.total_changes as f64) * 100.0
                } else {
                    0.0
                };

                ContributorStats {
                    user: c.git_user_email.clone(), // Using email as user identifier
                    email: c.git_user_email.clone(),
                    change_count: c.change_count,
                    percentage,
                }
            })
            .collect();

        let response_stats = SymbolStats {
            total_changes: stats.total_changes,
            contributors: contributor_stats,
            high_churn: stats.high_churn,
            bug_rate: stats.bug_rate,
            avg_complexity: stats.avg_complexity,
        };

        let response = BlameResponse {
            success: true,
            symbol_name: self.symbol.clone(),
            file_path: self.file.clone(),
            history: versions,
            stats: Some(response_stats),
        };

        if global_opts.json {
            println!("{}", serde_json::to_string_pretty(&response)?);
        } else {
            response.text()?;
        }

        Ok(())
    }
}
