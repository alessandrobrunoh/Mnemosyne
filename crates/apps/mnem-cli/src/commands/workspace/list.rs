use crate::commands::Command;
use crate::ui::Layout;
use crate::utils;
use anyhow::Result;
use mnem_core::{env::get_base_dir, storage::registry::ProjectRegistry};
use std::env;
use std::path::Path;

#[derive(Debug)]
pub struct ListCommand;

impl Command for ListCommand {
    fn name(&self) -> &str {
        "list"
    }

    fn usage(&self) -> &str {
        "[--limit <n>]"
    }

    fn description(&self) -> &str {
        "List all tracked projects"
    }

    fn group(&self) -> &str {
        "Workspace"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let common_args = utils::parse_common_args(args);
        let layout = Layout::new();

        let base_dir = get_base_dir()?;
        let registry = ProjectRegistry::new(&base_dir)?;
        let mut projects = registry.list_projects();
        projects.retain(|p| !p.name.is_empty() && p.name != "unknown" && p.path != "/");
        projects.sort_by(|a, b| b.last_open.cmp(&a.last_open));

        let cwd = env::current_dir()?;

        if projects.is_empty() {
            layout.warning("No projects tracked yet. Run `mnem` in a project directory to start.");
            return Ok(());
        }

        let total_count = projects.len();
        projects.truncate(common_args.limit);

        layout.header("TRACKED PROJECTS");
        layout.section_start("pr", "Tracked Projects");

        for (idx, p) in projects.iter().enumerate() {
            let id_tag = format!("{:02}", idx);
            let project_id_short = &p.id[..8.min(p.id.len())];

            let project_name = Path::new(&p.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let p_path = Path::new(&p.path);
            let is_current = cwd.starts_with(p_path) || p_path.starts_with(&cwd);

            let bullet = if is_current { "●" } else { "•" };

            let file_url = format!("file://{}", p.path);
            let interactive_id = format!(
                "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
                file_url, project_id_short
            );
            let dot = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", file_url, bullet);

            let content = format!("{}  {}  {}", project_name, dot, interactive_id);
            layout.row_list(&id_tag, &content);
        }
        layout.section_end();

        if total_count > projects.len() {
            layout.item_simple(&format!(
                "... and {} more projects. Use --limit <n> to see all.",
                total_count - projects.len()
            ));
        }

        layout.footer("Click a project ID to open it in your OS or type 'mnem <id>' for info.");
        Ok(())
    }
}
