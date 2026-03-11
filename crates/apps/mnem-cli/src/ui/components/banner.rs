use crate::theme::Theme;
use crossterm::style::Stylize;

/// A prominent banner component for headers or branding
#[derive(Debug, Clone)]
pub struct Banner {
    theme: Theme,
}

impl Banner {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn show(&self, text: &str) {
        let width = 60;
        let line = "━".repeat((width - text.len() - 2) / 2);
        println!();
        println!(
            "{} {} {}",
            line.clone().with(self.theme.border),
            text.with(self.theme.accent).bold(),
            line.with(self.theme.border)
        );
        println!();
    }
}
