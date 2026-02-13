use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;

#[derive(Debug)]
pub struct SetupProtocolCommand;

impl Command for SetupProtocolCommand {
    fn name(&self) -> &str {
        "setup-protocol"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Register Mnemosyne protocol handler (macOS only)"
    }

    fn group(&self) -> &str {
        "Maintenance"
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let layout = Layout::new();
        #[cfg(target_os = "macos")]
        {
            layout.section_start("os", "Protocol Registration");
            match crate::os::macos::install_url_handler() {
                Ok(_) => {
                    layout.item_simple(&format!(
                        "{} Protocol {} registered successfully!",
                        "√".green(),
                        "mnem://".bold().cyan()
                    ));
                    layout.item_simple("Location: ~/Applications/MnemHandler.app");
                    layout.footer("You can now Shift+Click hashes in your terminal.");
                }
                Err(e) => {
                    layout.item_simple(&format!("{} Failed: {}", "✘".red(), e));
                }
            }
            layout.section_end();
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("Protocol setup is currently only supported on macOS.");
        }
        Ok(())
    }
}
