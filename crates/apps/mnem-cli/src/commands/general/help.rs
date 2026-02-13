use crate::commands::Command;
use crate::i18n;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;

#[derive(Debug)]
pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Show help information for all commands"
    }

    fn group(&self) -> &str {
        "General"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["--help", "-h"]
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let msg = i18n::current();
        let layout = Layout::new();
        let all_commands = crate::commands::get_all();

        layout.section_start(
            "mn",
            &format!("{} [v{}]", msg.app_name(), env!("CARGO_PKG_VERSION")),
        );
        layout.item_simple(&msg.tagline().italic().dark_grey().to_string());

        let groups = [
            ("General", msg.core_ops_header().bold().cyan().to_string()),
            ("Daemon", "DAEMON".bold().cyan().to_string()),
            (
                "Workspace",
                msg.project_history_header().bold().cyan().to_string(),
            ),
            (
                "Git Integration",
                "INTEGRATIONS".bold().yellow().to_string(),
            ),
            ("Files", "FILES".bold().cyan().to_string()),
            (
                "Maintenance",
                msg.maintenance_header().bold().cyan().to_string(),
            ),
            ("Apps", "APPLICATIONS".bold().cyan().to_string()),
        ];

        for (group_id, group_header) in groups {
            layout.item_simple("");
            layout.item_simple(&group_header);

            for cmd in all_commands.iter().filter(|c| c.group() == group_id) {
                let name = if cmd.usage().is_empty() {
                    cmd.name().to_string()
                } else {
                    format!("{} {}", cmd.name(), cmd.usage())
                };
                layout.row_list(name.green().to_string().as_str(), cmd.description());
            }
        }

        layout.section_end();
        let footer_text = format!(
            "{}: https://github.com/alessandrobrunoh/mnemosyne",
            msg.learn_more()
        );
        layout.footer(&footer_text);

        Ok(())
    }
}
