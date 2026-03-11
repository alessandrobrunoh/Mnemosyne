use anyhow::Result;
use std::path::PathBuf;

/// Global options passed to all commands
#[derive(Clone, Debug)]
pub struct GlobalOptions {
    /// Project path override
    pub project: Option<PathBuf>,
    /// Output in JSON format
    pub json: bool,
}

impl GlobalOptions {
    /// Create a new GlobalOptions instance
    pub fn new(project: Option<PathBuf>, json: bool) -> Self {
        Self { project, json }
    }
}

/// Strategy trait for command execution
///
/// Each command implements this trait to provide its execution logic.
/// This allows for:
/// - Separation of command definition and execution
/// - Easy testing of individual commands
/// - Consistent error handling across all commands
pub trait CommandStrategy {
    /// Execute the command with the given global options
    fn execute(&self, global_opts: &GlobalOptions) -> Result<()>;
}
