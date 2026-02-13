use crate::commands;
use crate::context::Context;
use anyhow::Result;

/// Command dispatcher that parses arguments and routes to appropriate command
pub struct Dispatcher;

impl Dispatcher {
    /// Dispatch a command based on the provided arguments
    pub fn dispatch(ctx: &mut Context, args: &[String]) -> Result<()> {
        // No command provided - use default behavior
        if args.len() <= 1 {
            return Self::default_behavior(ctx, args);
        }

        let cmd_name = args[1].as_str();
        let commands = commands::get_all();

        // Check if it's a known command
        if let Some(cmd) = commands
            .iter()
            .find(|c| c.name() == cmd_name || c.aliases().contains(&cmd_name))
        {
            // Logic for context-dependent commands
            let context_dependent = matches!(
                cmd_name,
                "log"
                    | "diff"
                    | "info"
                    | "restore"
                    | "checkpoint"
                    | "tui"
                    | "search"
                    | "history"
                    | "timeline"
                    | "statistics"
                    | "stats"
            );

            if context_dependent {
                if let Some(project) = ctx.tracked_project.clone() {
                    ctx.ensure_watching(&project)?;
                }
            }

            return cmd.execute(args);
        }

        // Unknown command - check if it's a project ID
        if Self::is_project_id(cmd_name) {
            if let Some(cmd) = commands.iter().find(|c| c.name() == "project") {
                return cmd.execute(args);
            }
        }

        // Error: command not found
        eprintln!(
            "Error: \"mnem {}\" is not a command. Type \"mnem help\" to see all available commands.",
            cmd_name
        );
        std::process::exit(1);
    }

    /// Default behavior when no command is provided
    fn default_behavior(ctx: &mut Context, args: &[String]) -> Result<()> {
        let commands = commands::get_all();

        if let Some(project) = ctx.tracked_project.clone() {
            // Ensure daemon is watching this project
            ctx.ensure_watching(&project)?;

            // Show history for the project root
            if let Some(cmd) = commands.iter().find(|c| c.name() == "history") {
                // We need to pass the project path as an override if we could...
                // But the Command trait execute only takes args.
                // For now, let's just run history normally, it will find the CWD project.
                return cmd.execute(&["mnem".to_string(), "history".to_string()]);
            }
        } else {
            // Show project list
            if let Some(cmd) = commands.iter().find(|c| c.name() == "list") {
                return cmd.execute(args);
            }
        }

        Ok(())
    }

    /// Check if a string looks like a project ID (hex string >= 4 chars)
    fn is_project_id(s: &str) -> bool {
        s.len() >= 4 && s.chars().all(|c| c.is_ascii_hexdigit())
    }
}
