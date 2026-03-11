use crate::theme::Theme;
use crossterm::style::Stylize;

/// Structured technical details in a two-column grid
#[derive(Debug, Clone)]
pub struct MetadataGrid {
    theme: Theme,
}

impl MetadataGrid {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, data: Vec<(&str, String)>) {
        for (key, value) in data {
            println!(
                "  {: <10} {} {}",
                key.with(self.theme.text_dim).bold(),
                "│".with(self.theme.border),
                value.with(self.theme.text_bright)
            );
        }
    }
}
