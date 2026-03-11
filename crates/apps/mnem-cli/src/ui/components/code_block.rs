use crate::theme::Theme;
use crossterm::style::Stylize;

/// Syntax preview with line numbers
#[derive(Debug, Clone)]
pub struct CodeBlock {
    theme: Theme,
}

impl CodeBlock {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, lines: &[&str], start_line: usize) {
        for (i, line) in lines.iter().enumerate() {
            let line_num = start_line + i;
            println!(
                "  {: >3} {} {}",
                line_num.to_string().with(self.theme.text_dim),
                "│".with(self.theme.border),
                line.with(self.theme.text)
            );
        }
    }
}
