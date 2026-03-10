use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::ui::{Layout, Presentable};

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

impl Presentable for TrackResponse {
    fn render_tui(&self) -> Result<()> {
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
                    theme.timeline_purple
                );
                layout.graph_file_change(&p.path, "root");
            }
            layout.graph_branch_end();

            // Pagination info
            let total_pages = (self.total_projects as f64 / self.limit as f64).ceil() as usize;
            println!();
            println!(
                "  {} {}/{}  {} {}  {} {}",
                "PAGE".with(theme.text_dim).bold(),
                self.page.to_string().with(theme.text_bright),
                total_pages.to_string().with(theme.text_dim),
                "TOTAL".with(theme.text_dim).bold(),
                self.total_projects.to_string().with(theme.text_bright),
                "STATUS".with(theme.text_dim).bold(),
                "Listing".with(theme.text_bright)
            );
        } else if let Some(current) = &self.current {
            if self.action == "remove" {
                layout.graph_branch_start("workspace: project tracking");
                layout.graph_node(
                    &current.id,
                    &current.name,
                    false,
                    "stopped tracking",
                    Some("x"),
                    theme.error
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
                    theme.success_bright
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
                theme.text_dim
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

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_track(list: bool, remove: bool, purge: bool, id: Option<String>, limit: usize, page: usize, json: bool) -> Result<()> {
    use mnem_core::client::daemon_running;
    use mnem_core::env::get_base_dir;
    use mnem_core::protocol::methods;
    use mnem_core::storage::registry::ProjectRegistry;

    let base_dir = get_base_dir()?;
    let mut registry = ProjectRegistry::new(&base_dir)?;

    if list {
        let all_projects = registry.list_projects();
        let total_projects = all_projects.len();
        let offset = (page.saturating_sub(1)) * limit;

        let paginated_projects = all_projects
            .into_iter()
            .skip(offset)
            .take(limit)
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
            limit,
            page,
            action: "list".to_string(),
        };

        if json {
            println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
        } else {
            response.render_tui()?;
        }
        return Ok(());
    }

    let cwd = std::env::current_dir()?;

    if remove || purge {
        let project_id = if let Some(ref custom_id) = id {
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
                registry.list_projects().into_iter()
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

                if purge {
                    let mnem_dir = cwd.join(".mnemosyne");
                    if mnem_dir.exists() {
                        if let Err(e) = std::fs::remove_dir_all(&mnem_dir) {
                            message = format!("Removed from tracking, but failed to delete .mnemosyne: {}", e);
                        } else {
                            message = "Project untracked and local history purged successfully".to_string();
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
            limit,
            page,
            action: "remove".to_string(),
        };

        if json {
            println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
        } else {
            response.render_tui()?;
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
            Ok(mut client) => match client.call(methods::PROJECT_RELOAD, serde_json::Value::Null) {
                Ok(_) => {
                    message = "Tracking started and daemon reloaded".to_string();
                }
                Err(e) => {
                    message = format!("Tracking started but could not reload daemon: {}", e);
                }
            },
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
        limit,
        page,
        action: "track".to_string(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}
