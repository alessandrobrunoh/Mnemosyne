use crate::theme::Theme;
use crossterm::style::Stylize;
use std::time::Duration;

/// A simple CLI spinner for background operations
#[derive(Debug, Clone)]
pub struct Spinner {
    theme: Theme,
    frames: Vec<&'static str>,
}

impl Spinner {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        }
    }

    /// Display a static spinner frame with a message
    pub fn frame(&self, step: usize, message: &str) {
        let frame = self.frames[step % self.frames.len()];
        print!(
            "\r{} {}",
            frame.with(self.theme.accent).bold(),
            message.with(self.theme.text)
        );
        use std::io::Write;
        std::io::stdout().flush().ok();
    }

    /// Clear the current spinner line
    pub fn clear(&self) {
        print!("\r\x1b[2K");
    }
}
