use crate::theme::Theme;
use crossterm::style::Stylize;

/// Interactive guide for keyboard shortcuts
#[derive(Debug, Clone)]
pub struct KeyHint {
    theme: Theme,
}

impl KeyHint {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn show(&self, hints: &[(&str, &str)]) {
        let mut row = String::new();
        for (i, (key, desc)) in hints.iter().enumerate() {
            if i > 0 {
                row.push_str("  ");
            }
            row.push_str(&format!(
                "[{}] {}",
                key.with(self.theme.accent).bold(),
                desc.with(self.theme.text_dim)
            ));
        }
        println!("  {}", row);
    }
}
