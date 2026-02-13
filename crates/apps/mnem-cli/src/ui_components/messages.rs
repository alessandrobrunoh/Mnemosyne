use crate::theme::Theme;
use crossterm::style::Stylize;
use mnem_macros::UiDebug;

/// Messages component for displaying user-facing messages
///
/// Provides styled message rendering with theme-aware coloring.
///
/// # Example
/// ```rust
/// use mnem_cli::ui_components::Messages;
/// use mnem_cli::theme::Theme;
///
/// let messages = Messages::new(Theme::default());
/// messages.success("Operation completed!");
/// messages.error("Something went wrong");
/// messages.warning("Be careful!");
/// messages.info("Here's some information");
/// ```
#[derive(Debug, Clone, UiDebug)]
pub struct Messages {
    theme: Theme,
}

impl Messages {
    /// Internal test rendering logic
    pub fn test_output(&self) {
        self.success("File saved successfully!");
        self.error("Failed to connect to daemon");
        self.warning("This operation cannot be undone");
        self.info("Daemon is running in background");
        self.debug("Loaded 5 projects from registry");
        self.prompt("Enter your choice");
        println!();
        self.confirm("Are you sure you want to continue?");
        self.hint("Use 'mnem help' to see all available commands");
        self.list_item("1", "First option", Some("active"));
    }

    /// Create a new messages component with the given theme
    /// Create a new messages component with the given theme
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Display a success message with checkmark icon
    ///
    /// # Example
    /// ```rust
    /// messages.success("File saved successfully!");
    /// ```
    pub fn success(&self, message: &str) {
        println!(
            "{} {}",
            "✓".with(self.theme.success),
            message.with(self.theme.success)
        );
    }

    /// Display an error message with cross icon
    ///
    /// # Example
    /// ```rust
    /// messages.error("Failed to connect to daemon");
    /// ```
    pub fn error(&self, message: &str) {
        eprintln!(
            "{} {}",
            "✗".with(self.theme.error),
            message.with(self.theme.error)
        );
    }

    /// Display a warning message with warning icon
    ///
    /// # Example
    /// ```rust
    /// messages.warning("This operation cannot be undone");
    /// ```
    pub fn warning(&self, message: &str) {
        println!(
            "{} {}",
            "⚠".with(self.theme.warning),
            message.with(self.theme.warning)
        );
    }

    /// Display an informational message with info icon
    ///
    /// # Example
    /// ```rust
    /// messages.info("Daemon is running in background");
    /// ```
    pub fn info(&self, message: &str) {
        println!(
            "{} {}",
            "ℹ".with(self.theme.accent),
            message.with(self.theme.accent)
        );
    }

    /// Display a debug message (dimmed)
    ///
    /// # Example
    /// ```rust
    /// messages.debug("Loaded 5 projects from registry");
    /// ```
    pub fn debug(&self, message: &str) {
        println!(
            "{} {}",
            "◇".with(self.theme.text_dim),
            message.with(self.theme.text_dim)
        );
    }

    /// Display a prompt to the user
    ///
    /// # Example
    /// ```rust
    /// messages.prompt("Enter project name:");
    /// ```
    pub fn prompt(&self, message: &str) {
        print!(
            "{} {}: ",
            "→".with(self.theme.accent),
            message.with(self.theme.active)
        );
        // Flush to ensure prompt appears before input
        use std::io::Write;
        std::io::stdout().flush().ok();
    }

    /// Display a confirmation question
    ///
    /// # Example
    /// ```rust
    /// let answer = messages.confirm("Are you sure you want to delete this project?");
    /// ```
    pub fn confirm(&self, message: &str) {
        println!(
            "{} {} [y/N]",
            "?".with(self.theme.warning),
            message.with(self.theme.warning)
        );
    }

    /// Display a hint/tip message
    ///
    /// # Example
    /// ```rust
    /// messages.hint("Use 'mnem help' to see all available commands");
    /// ```
    pub fn hint(&self, message: &str) {
        println!();
        println!(
            "  {} {} {}",
            "💡".with(self.theme.warning),
            "Hint:".with(self.theme.text_dim).bold(),
            message.with(self.theme.text_dim).italic()
        );
    }

    /// Display a formatted list item
    ///
    /// # Example
    /// ```rust
    /// messages.list_item("1", "First project", "active");
    /// ```
    pub fn list_item(&self, index: &str, content: &str, status: Option<&str>) {
        let status_colored = match status {
            Some(s) => s.with(self.theme.active).to_string(),
            None => "".to_string(),
        };

        println!(
            "  {: <12} {} {}",
            index.with(self.theme.text_dim),
            content.with(self.theme.text),
            status_colored
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_creation() {
        let theme = Theme::default();
        let messages = Messages::new(theme);

        // Test that methods don't panic
        messages.success("Test success");
        messages.error("Test error");
        messages.warning("Test warning");
        messages.info("Test info");
        messages.debug("Test debug");
        messages.prompt("Test prompt");
        messages.confirm("Test confirm");
        messages.hint("Test hint");
        messages.list_item("1", "Item", Some("active"));
    }
}
