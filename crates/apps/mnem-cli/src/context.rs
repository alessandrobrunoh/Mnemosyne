use anyhow::Result;
use crossterm::style::Stylize;
use mnem_core::models::Project;
use mnem_core::storage::registry::ProjectRegistry;
use mnem_core::{client::DaemonClient, env::get_base_dir, protocol::methods};
use serde_json::json;
use std::path::{Path, PathBuf};

/// Application context containing detected project and environment information
pub struct Context {
    /// Current working directory
    pub cwd: PathBuf,
    /// Mnemosyne base directory
    pub base_dir: PathBuf,
    /// Tracked project if current directory is within a project
    pub tracked_project: Option<Project>,
    /// Daemon client (initialized on demand)
    daemon_client: Option<DaemonClient>,
    /// UI theme for terminal output
    pub theme: crate::theme::Theme,
}

impl Context {
    /// Create a new context by detecting the current project
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let base_dir = get_base_dir()?;

        // Load and sort projects by path length (longest first = deepest match)
        let registry = ProjectRegistry::new(&base_dir)?;
        let mut projects = registry.list_projects();
        projects.sort_by(|a, b| b.path.len().cmp(&a.path.len()));

        // Find project that contains current directory
        let tracked_project = projects.into_iter().find(|p| {
            let p_path = Path::new(&p.path);

            // Skip system/root directories to avoid false matches
            if p.path == "/"
                || (p.path.starts_with("/Users") && p.path.split('/').count() <= 3)
                || p.name == "unknown"
            {
                return false;
            }

            cwd == p_path || cwd.starts_with(p_path)
        });

        Ok(Self {
            cwd,
            base_dir,
            tracked_project,
            daemon_client: None,
            theme: crate::theme::Theme::default(),
        })
    }

    /// Get a reference to tracked project if any
    pub fn project(&self) -> Option<&Project> {
        self.tracked_project.as_ref()
    }

    /// Get the daemon client, initializing it if necessary
    pub fn daemon(&mut self) -> Result<&mut DaemonClient> {
        if self.daemon_client.is_none() {
            // Ensure daemon is running
            let _ = mnem_core::client::ensure_daemon();
            self.daemon_client = Some(DaemonClient::connect()?);
        }

        Ok(self
            .daemon_client
            .as_mut()
            .expect("daemon_client was just initialized"))
    }

    /// Ensure daemon is watching given project
    pub fn ensure_watching(&mut self, project: &Project) -> Result<()> {
        let _ = mnem_core::client::ensure_daemon();

        let client = self.daemon()?;
        let _ = client.call(
            methods::PROJECT_WATCH,
            json!({"project_path": project.path}),
        );

        Ok(())
    }

    /// Check if the current directory is within a tracked project
    pub fn is_in_project(&self) -> bool {
        self.tracked_project.is_some()
    }

    /// Get the project path if in a project, otherwise return current directory
    pub fn project_path_or_cwd(&self) -> &Path {
        self.tracked_project
            .as_ref()
            .map(|p| Path::new(p.path.as_str()))
            .unwrap_or(&self.cwd)
    }

    /// Get reference to theme
    pub fn theme(&self) -> &crate::theme::Theme {
        &self.theme
    }

    /// Get mutable reference to theme
    pub fn theme_mut(&mut self) -> &mut crate::theme::Theme {
        &mut self.theme
    }

    /// Get accent color for highlights
    pub fn accent(&self) -> crossterm::style::Color {
        self.theme.accent
    }

    /// Get error color
    pub fn error_color(&self) -> crossterm::style::Color {
        self.theme.error
    }

    /// Get success color
    pub fn success_color(&self) -> crossterm::style::Color {
        self.theme.success
    }

    /// Get warning color
    pub fn warning_color(&self) -> crossterm::style::Color {
        self.theme.warning
    }

    /// Get border color
    pub fn border_color(&self) -> crossterm::style::Color {
        self.theme.border
    }

    // =============================================================================
    // UTILITY METHODS FOR FORMATTED OUTPUT
    // =============================================================================

    /// Print success message with theme color
    pub fn print_success(&self, message: &str) {
        println!("{} {}", "✓".with(self.success_color()), message);
    }

    /// Print error message with theme color
    pub fn print_error(&self, message: &str) {
        eprintln!("{} {}", "✘".with(self.error_color()), message);
    }

    /// Print warning message with theme color
    pub fn print_warning(&self, message: &str) {
        println!("{} {}", "⚠".with(self.warning_color()), message);
    }

    /// Print info message with accent color
    pub fn print_info(&self, message: &str) {
        println!("{} {}", "ℹ".with(self.accent()), message);
    }

    /// Format text with success color
    pub fn format_success(&self, text: &str) -> String {
        text.with(self.success_color()).to_string()
    }

    /// Format text with error color
    pub fn format_error(&self, text: &str) -> String {
        text.with(self.error_color()).to_string()
    }

    /// Format text with warning color
    pub fn format_warning(&self, text: &str) -> String {
        text.with(self.warning_color()).to_string()
    }

    /// Format text with accent color
    pub fn format_accent(&self, text: &str) -> String {
        text.with(self.accent()).to_string()
    }
}
