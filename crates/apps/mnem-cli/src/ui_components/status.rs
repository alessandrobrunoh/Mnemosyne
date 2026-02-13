use crate::theme::Theme;
use crossterm::style::Stylize;
use mnem_macros::UiDebug;

/// Status component for displaying status messages
///
/// Provides methods for rendering success, error, warning, and info messages
/// with theme-aware coloring.
#[derive(Debug, Clone, UiDebug)]
pub struct Status {
    theme: Theme,
}

impl Status {
    /// Internal test rendering logic
    pub fn test_output(&self) {
        self.success("Operation completed successfully!");
        self.error("An error occurred while processing");
        self.warning("This is a warning message");
        self.info("This is an informational message");
    }

    /// Create a new status component with the given theme
    /// Create a new status component with the given theme
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Display a success message
    ///
    /// # Example
    /// ```rust
    /// use mnem_cli::ui_components::Status;
    /// use mnem_cli::theme::Theme;
    ///
    /// let status = Status::new(Theme::default());
    /// status.success("Operation completed successfully!");
    /// ```
    pub fn success(&self, message: &str) {
        println!(
            "{} {}",
            "✓".with(self.theme.success),
            message.with(self.theme.success)
        );
    }

    /// Display an error message
    ///
    /// # Example
    /// ```rust
    /// status.error("Failed to connect to daemon");
    /// ```
    pub fn error(&self, message: &str) {
        eprintln!(
            "{} {}",
            "✗".with(self.theme.error),
            message.with(self.theme.error)
        );
    }

    /// Display a warning message
    ///
    /// # Example
    /// ```rust
    /// status.warning("This operation cannot be undone");
    /// ```
    pub fn warning(&self, message: &str) {
        println!(
            "{} {}",
            "⚠ ".with(self.theme.warning),
            message.with(self.theme.warning)
        );
    }

    /// Display an info message
    ///
    /// # Example
    /// ```rust
    /// status.info("Daemon is running in background");
    /// ```
    pub fn info(&self, message: &str) {
        println!(
            "{} {}",
            "ℹ ".with(self.theme.accent),
            message.with(self.theme.accent)
        );
    }
}
