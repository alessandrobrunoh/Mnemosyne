use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::{Layout, List, PaginationInfo};
use crate::ui::presentable::Renderable;
use anyhow::Result;
use clap::Args;
use crossterm::style::Stylize;
use serde::Serialize;

#[derive(Serialize)]
pub struct TrackResponse {
    pub success: bool,
    pub projects: Vec<ProjectInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<ProjectInfo>,
    pub message: String,
    pub total_projects: usize,
    pub limit: usize,
    pub page: usize,
    #[serde(skip)]
    pub action: String,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub id: String,
}

impl Renderable for TrackResponse {
    fn text(&self) -> Result<()> {
        let layout = Layout::new();
        let theme = layout.theme();
        use crossterm::style::Stylize;

        if !self.projects.is_empty() {
            layout.graph_branch_start("workspace: tracked projects");
            for p in &self.projects {
                layout.graph_node(
                    &p.id,
                    &p.name,
                    false,
                    "active",
                    Some("•"),
                    theme.timeline_purple,
                );
                layout.graph_file_change(&p.path, "root");
            }
            layout.graph_branch_end();

            // Pagination info
            let list = List::new(theme.clone());
            let info = PaginationInfo::new(self.page, self.total_projects, self.limit)
                .with_info("STATUS".to_string(), "Listing".to_string());
            list.pagination(&info);
        } else if let Some(current) = &self.current {
            if self.action == "remove" {
                layout.graph_branch_start("workspace: project tracking");
                layout.graph_node(
                    &current.id,
                    &current.name,
                    false,
                    "stopped tracking",
                    Some("x"),
                    theme.error,
                );
                layout.graph_file_change(&current.path, "root");
                layout.graph_branch_end();
                layout.empty();
                layout.badge_success("OK", &self.message);
            } else {
                layout.graph_branch_start("workspace: project tracking");
                layout.graph_node(
                    &current.id,
                    &current.name,
                    true,
                    "now tracking",
                    Some("✓"),
                    theme.success_bright,
                );
                layout.graph_file_change(&current.path, "root");
                layout.graph_branch_end();
                layout.empty();
                layout.badge_success("OK", &self.message);
            }
        } else {
            layout.graph_branch_start("workspace");
            layout.graph_node(
                "EMPTY",
                "STATUS",
                false,
                "no projects",
                None,
                theme.text_dim,
            );
            layout.graph_branch_end();
            layout.empty();
            if self.action == "remove" {
                layout.success_bright(&self.message);
            } else {
                layout.info("Use 'mnem track' in a project directory to start tracking");
            }
        }

        Ok(())
    }
}

/// Track and manage projects
#[derive(Args, Clone, Debug)]
pub struct TrackCommand {
    /// List all tracked projects
    #[arg(short, long)]
    pub list: bool,

    /// Remove project from tracking
    #[arg(short, long)]
    pub remove: bool,

    /// Remove project and delete local history
    #[arg(short, long)]
    pub purge: bool,

    /// Project ID to remove
    #[arg(short, long)]
    pub id: Option<String>,

    /// Maximum number of results
    #[arg(short, long, default_value = "20")]
    pub limit: usize,

    /// Page number
    #[arg(short = 'P', long, default_value = "1")]
    pub page: usize,
}

impl CommandStrategy for TrackCommand {
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()> {
        use mnem_core::client::daemon_running;
        use mnem_core::env::get_base_dir;
        use mnem_core::protocol::methods;
        use mnem_core::storage::registry::ProjectRegistry;

        let base_dir = get_base_dir()?;
        let mut registry = ProjectRegistry::new(&base_dir)?;

        if self.list {
            let all_projects = registry.list_projects();
            let total_projects = all_projects.len();
            let offset = (self.page.saturating_sub(1)) * self.limit;

            let paginated_projects = all_projects
                .into_iter()
                .skip(offset)
                .take(self.limit)
                .map(|p| ProjectInfo {
                    name: p.name,
                    path: p.path,
                    id: p.id,
                })
                .collect();

            let response = TrackResponse {
                success: true,
                projects: paginated_projects,
                current: None,
                message: "Listing tracked projects".to_string(),
                total_projects,
                limit: self.limit,
                page: self.page,
                action: "list".to_string(),
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response.json()?)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        let cwd = std::env::current_dir()?;

        if self.remove || self.purge {
            let project_id = if let Some(ref custom_id) = self.id {
                Some(custom_id.clone())
            } else {
                let tracked_file = cwd.join(".mnemosyne").join("tracked");
                if tracked_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&tracked_file) {
                        content
                            .lines()
                            .find(|l| l.starts_with("project_id:"))
                            .and_then(|l| l.split(':').nth(1))
                            .map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                } else {
                    // Not tracked locally, try finding by path
                    registry
                        .list_projects()
                        .into_iter()
                        .find(|p| p.path == cwd.to_string_lossy().to_string())
                        .map(|p| p.id)
                }
            };

            let mut project_info = None;
            let mut message = "Project not found or not tracked".to_string();
            let mut success = false;

            if let Some(pid) = project_id {
                if let Ok(Some(removed_project)) = registry.remove(&pid) {
                    project_info = Some(ProjectInfo {
                        name: removed_project.name,
                        path: removed_project.path,
                        id: removed_project.id,
                    });

                    success = true;
                    message = "Project removed from tracking".to_string();

                    if self.purge {
                        let mnem_dir = cwd.join(".mnemosyne");
                        if mnem_dir.exists() {
                            if let Err(e) = std::fs::remove_dir_all(&mnem_dir) {
                                message = format!(
                                    "Removed from tracking, but failed to delete .mnemosyne: {}",
                                    e
                                );
                            } else {
                                message = "Project untracked and local history purged successfully"
                                    .to_string();
                            }
                        }
                    }

                    // Notify daemon to reload projects
                    if daemon_running() {
                        if let Ok(mut client) = mnem_core::client::DaemonClient::connect() {
                            let _ = client.call(methods::PROJECT_RELOAD, serde_json::Value::Null);
                        }
                    }
                }
            }

            let response = TrackResponse {
                success,
                projects: vec![],
                current: project_info,
                message,
                total_projects: 0,
                limit: self.limit,
                page: self.page,
                action: "remove".to_string(),
            };

            if global_opts.json {
                println!("{}", serde_json::to_string_pretty(&response.json()?)?);
            } else {
                response.text()?;
            }
            return Ok(());
        }

        if !daemon_running() {
            mnem_core::client::ensure_daemon()?;
        }

        let project = registry.get_or_create(&cwd)?;
        let project_info = ProjectInfo {
            name: project.name.clone(),
            path: project.path.clone(),
            id: project.id.clone(),
        };

        let mut message = "Tracking started".to_string();

        if daemon_running() {
            match mnem_core::client::DaemonClient::connect() {
                Ok(mut client) => {
                    match client.call(methods::PROJECT_RELOAD, serde_json::Value::Null) {
                        Ok(_) => {
                            message = "Tracking started and daemon reloaded".to_string();
                        }
                        Err(e) => {
                            message =
                                format!("Tracking started but could not reload daemon: {}", e);
                        }
                    }
                }
                Err(e) => {
                    message = format!("Tracking started but could not connect to daemon: {}", e);
                }
            }
        }

        let response = TrackResponse {
            success: true,
            projects: vec![],
            current: Some(project_info),
            message,
            total_projects: 0,
            limit: self.limit,
            page: self.page,
            action: "track".to_string(),
        };

        if global_opts.json {
            println!("{}", serde_json::to_string_pretty(&response.json()?)?);
        } else {
            response.text()?;
        }

        Ok(())
    }
}
