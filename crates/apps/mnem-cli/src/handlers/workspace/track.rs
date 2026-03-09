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

        if !self.projects.is_empty() {
            layout.header_dashboard("TRACKED PROJECTS");
            for p in &self.projects {
                layout.bullet_purple(&p.name);
                layout.row_file_path(&p.path);
            }
            layout.section_end();
        }

        if let Some(current) = &self.current {
            layout.header_dashboard("PROJECT TRACKED");
            layout.success_bright(&format!("✓ Now tracking: {}", current.name));
            layout.empty();
            layout.row_labeled("◫", "Path", &current.path);
            layout.row_labeled("◆", "ID", &current.id);
            layout.empty();
            layout.badge_success("OK", &self.message);
        } else if self.projects.is_empty() {
            layout.warning("No tracked projects.");
            layout.empty();
            layout.info("Use 'mnem track' in a project directory to start tracking");
        }

        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_track(list: bool, _remove: bool, _id: Option<String>, json: bool) -> Result<()> {
    use mnem_core::client::daemon_running;
    use mnem_core::env::get_base_dir;
    use mnem_core::protocol::methods;
    use mnem_core::storage::registry::ProjectRegistry;

    let base_dir = get_base_dir()?;
    let mut registry = ProjectRegistry::new(&base_dir)?;

    if list {
        let projects = registry
            .list_projects()
            .into_iter()
            .map(|p| ProjectInfo {
                name: p.name,
                path: p.path,
                id: p.id,
            })
            .collect();

        let response = TrackResponse {
            success: true,
            projects,
            current: None,
            message: "Listing tracked projects".to_string(),
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
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response.render_json()?)?);
    } else {
        response.render_tui()?;
    }

    Ok(())
}
