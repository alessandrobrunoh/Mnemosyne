use crate::commands::Command;
use crate::i18n;
use crate::ui::Layout;
use anyhow::Result;
use crossterm::style::Stylize;

#[derive(Debug)]
pub struct VersionCommand;

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Show version information and ASCII banner"
    }

    fn group(&self) -> &str {
        "General"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["--version", "-v"]
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let msg = i18n::current();
        let layout = Layout::new();
        println!("");
        println!("{}", " ███╗   ███╗███╗   ██╗███████╗███╗   ███╗".cyan());
        println!("{}", " ████╗ ████║████╗  ██║██╔════╝████╗ ████║".cyan());
        println!("{}", " ██╔████╔██║██╔██╗ ██║█████╗  ██╔████╔██║".cyan());
        println!("{}", " ██║╚██╔╝██║██║╚██╗██║██╔══╝  ██║╚██╔╝██║".cyan());
        println!("{}", " ██║ ╚═╝ ██║██║ ╚████║███████╗██║ ╚═╝ ██║".cyan());
        println!("{}", " ╚═╝     ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚═╝".cyan());

        layout.section_start("v", &format!("mnemosyne v{}", env!("CARGO_PKG_VERSION")));
        layout.item_simple(&msg.tagline().italic().dark_grey().to_string());
        layout.section_end();
        Ok(())
    }
}
