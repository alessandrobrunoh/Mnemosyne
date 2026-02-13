use crate::commands::Command;
use crate::theme::Theme;
use crate::ui_components::*;
use anyhow::Result;

#[derive(Debug)]
pub struct ComponentsCommand;

impl Command for ComponentsCommand {
    fn name(&self) -> &str {
        "components"
    }

    fn usage(&self) -> &str {
        "[<component_name>]"
    }

    fn description(&self) -> &str {
        "UI component library preview and testing tool"
    }

    fn group(&self) -> &str {
        "Apps"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let theme = Theme::default();
        let name = args.get(2).map(|s| s.as_str()).unwrap_or("all");

        // Register all components that implement UIComponent
        let components: Vec<Box<dyn UIComponent>> = vec![
            Box::new(crate::ui::Layout::new()),
            Box::new(List::new(theme.clone())),
            Box::new(Status::new(theme.clone())),
            Box::new(Messages::new(theme.clone())),
            Box::new(Elements::new(theme.clone())),
            Box::new(crate::ui::TsHighlighter::new()),
        ];

        if name == "all" {
            println!();
            println!("═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═");
            println!("    TESTING ALL UI COMPONENTS");
            println!("═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ═ ");
            println!();

            for comp in &components {
                comp.render_test();
            }

            println!();
            println!("✓ All components tested successfully!");
            println!();
        } else {
            let comp = components
                .iter()
                .find(|c| c.name() == name)
                .ok_or_else(|| {
                    println!("Unknown component: {}", name);
                    println!();
                    println!("Available components:");
                    for c in &components {
                        println!("  {}", c.name());
                    }
                    println!("  all      - Test all components");
                    println!();
                    println!("Usage: mnem components <component_name>");
                    std::process::exit(1);
                })
                .unwrap();

            comp.render_test();
        }

        Ok(())
    }
}
