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
            layout.info("Use 'mnem track' in a project directory to start tracking");
        }

        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_track(list: bool, _remove: bool, _id: Option<String>, limit: usize, page: usize, json: bool) -> Result<()> {
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
        };

        if json {
            println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
        } else {
            response.render_tui()?;
        }
        return Ok(());
    }

    let cwd = std::env::current_dir()?;

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
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}
