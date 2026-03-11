use crate::theme::Theme;
use crossterm::style::Stylize;

/// A framed card component for summarizing entities
#[derive(Debug, Clone)]
pub struct Card {
    theme: Theme,
    title: String,
    width: usize,
}

impl Card {
    pub fn new(theme: Theme, title: &str) -> Self {
        Self {
            theme,
            title: title.to_string(),
            width: 44,
        }
    }

    pub fn render(&self, content: Vec<(&str, String)>) {
        let top = format!("┌{:─<width$}┐", "", width = self.width - 2);
        println!("  {}", top.with(self.theme.border));

        // Title line
        let title_icon = "󱂵";
        let title_line = format!(
            "│ {} {: <width$} │",
            title_icon,
            self.title,
            width = self.width - 7
        );
        println!("  {}", title_line.with(self.theme.accent).bold());

        // Separator
        let sep = format!(
            "│ {: <width$} │",
            "─".repeat(self.width - 4),
            width = self.width - 4
        );
        println!("  {}", sep.with(self.theme.border));

        // Content lines
        for (label, value) in content {
            let line = format!(
                "│ {}: {: <width$} │",
                label,
                value,
                width = self.width - label.len() - 6
            );
            println!("  {}", line.with(self.theme.text));
        }

        let bottom = format!("└{:─<width$}┘", "", width = self.width - 2);
        println!("  {}", bottom.with(self.theme.border));
    }
}
