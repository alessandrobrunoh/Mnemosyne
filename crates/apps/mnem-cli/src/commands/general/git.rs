
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::ui::{Layout, Presentable};

#[derive(Serialize)]
pub struct GitCommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub files: String,
}

#[derive(Serialize)]
pub struct GitResponse {
    pub success: bool,
    pub commits: Vec<GitCommitInfo>,
}

impl Presentable for GitResponse {
    fn render_tui(&self) -> Result<()> {
        let layout = Layout::new();
        layout.header_dashboard("GIT COMMITS");
        layout.section_branch("gt", "Recent Commits");

        if self.commits.is_empty() {
            layout.item_simple("No git commits found.");
        } else {
            for commit in &self.commits {
                layout.row_history_compact(
                    &commit.hash[..8],
                    "G",
                    &commit.message,
                    &commit.timestamp,
                    false,
                    None,
                );
            }
        }
        layout.section_end();
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_git(commits: bool, log: bool, _hook: bool, json: bool) -> Result<()> {
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::Repository;

    let base_dir = get_base_dir()?;
    let cwd = std::env::current_dir()?;
    let repo = Repository::open(base_dir, cwd)?;

    if commits || log {
        let git_commits = repo.list_commits()?;
        let commits_info: Vec<GitCommitInfo> = git_commits
            .into_iter()
            .map(|(hash, message, author, timestamp, files)| GitCommitInfo {
                hash,
                message,
                author,
                timestamp,
                files: files.to_string(),
            })
            .collect();

        let response = GitResponse {
            success: true,
            commits: commits_info,
        };

        if json {
            println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
        } else if log {
            println!("Git Log:");
            println!("─");
            for commit in &response.commits {
                println!("{}  {}  {}", &commit.hash[..8], commit.timestamp, commit.message);
            }
        } else {
            response.render_tui()?;
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::json!({ "success": false, "error": "No git action specified" }));
    } else {
        println!("Usage:");
        println!("  mnem git --commits   # list commits");
        println!("  mnem git --log      # git log");
        println!("  mnem git --hook     # install hook");
    }

    Ok(())
}
