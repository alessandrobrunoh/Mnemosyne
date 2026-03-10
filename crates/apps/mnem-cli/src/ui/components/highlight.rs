use crate::theme::Theme;
use crossterm::style::Stylize;

/// Utility for highlighting patterns in text
#[derive(Debug, Clone)]
pub struct Highlight {
    theme: Theme,
}

impl Highlight {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Highlight all occurrences of a pattern in a string
    pub fn text(&self, input: &str, pattern: &str) -> String {
        if pattern.is_empty() {
            return input.to_string();
        }
        
        input.replace(
            pattern,
            &pattern.with(self.theme.active).bold().underlined().to_string()
        )
    }
}
