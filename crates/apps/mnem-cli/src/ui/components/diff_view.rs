use crate::theme::Theme;
use crossterm::style::Stylize;

/// A component for rendering unified or side-by-side diffs
#[derive(Debug, Clone)]
pub struct DiffView {
    theme: Theme,
}

impl DiffView {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render_line(&self, line: &str, change_type: &str) {
        match change_type {
            "+" => {
                println!(
                    "{} {}",
                    "+".with(self.theme.success).bold(),
                    line.with(self.theme.success)
                );
            }
            "-" => {
                println!(
                    "{} {}",
                    "-".with(self.theme.error).bold(),
                    line.with(self.theme.error)
                );
            }
            _ => {
                println!("  {}", line.with(self.theme.text_dim));
            }
        }
    }

    pub fn header(&self, file_path: &str) {
        println!(
            "{} {}",
            "DIFF".with(self.theme.accent).bold(),
            file_path.with(self.theme.text_bright).underlined()
        );
    }
}
