use anyhow::Result;

/// Trait for all CLI commands
pub trait Command: std::fmt::Debug {
    /// Command name (e.g. "log")
    fn name(&self) -> &str;

    /// Command usage string (e.g. "<file_path>")
    fn usage(&self) -> &str;

    /// Short description of the command
    fn description(&self) -> &str;

    /// Command group for help organization
    fn group(&self) -> &str {
        "Other"
    }

    /// Command aliases (e.g. ["grep"])
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    /// Run the command logic
    fn execute(&self, args: &[String]) -> Result<()>;
}

pub mod apps;
pub mod daemon;
pub mod files;
pub mod general;
pub mod git;
pub mod maintenance;
pub mod workspace;

/// Get all available commands organized by group
pub fn get_all() -> Vec<Box<dyn Command>> {
    let mut commands: Vec<Box<dyn Command>> = Vec::new();

    // General
    commands.push(Box::new(general::HelpCommand));
    commands.push(Box::new(general::VersionCommand));

    // Daemon
    commands.push(Box::new(daemon::StartCommand));
    commands.push(Box::new(daemon::StopCommand));
    commands.push(Box::new(daemon::StatusCommand));
    commands.push(Box::new(daemon::McpCommand));

    // Workspace
    commands.push(Box::new(workspace::ListCommand));
    commands.push(Box::new(workspace::WatchCommand));
    commands.push(Box::new(workspace::ForgetCommand));
    commands.push(Box::new(workspace::ProjectCommand));
    commands.push(Box::new(workspace::HistoryCommand));
    commands.push(Box::new(workspace::StatisticsCommand));
    commands.push(Box::new(workspace::CheckpointCommand));
    commands.push(Box::new(workspace::CheckpointsCommand));
    commands.push(Box::new(workspace::CheckpointInfoCommand));

    // Git Integration
    commands.push(Box::new(git::CommitsCommand));
    commands.push(Box::new(git::CommitInfoCommand));
    commands.push(Box::new(git::LogCommitsCommand));
    commands.push(Box::new(git::GitEventCommand));
    commands.push(Box::new(git::GitHookCommand));

    // Files
    commands.push(Box::new(files::LogCommand));
    commands.push(Box::new(files::DiffCommand));
    commands.push(Box::new(files::InfoCommand));
    commands.push(Box::new(files::OpenCommand));
    commands.push(Box::new(files::CatCommand));
    commands.push(Box::new(files::RestoreCommand));
    commands.push(Box::new(files::SearchCommand));
    commands.push(Box::new(files::TimelineCommand));

    // Maintenance
    commands.push(Box::new(maintenance::ConfigCommand));
    commands.push(Box::new(maintenance::SetupProtocolCommand));
    commands.push(Box::new(maintenance::GcCommand));

    // Apps
    commands.push(Box::new(apps::TuiCommand));
    commands.push(Box::new(apps::ComponentsCommand));

    commands
}
