use crate::theme::Theme;
use crossterm::style::Stylize;

/// Navigational context component
#[derive(Debug, Clone)]
pub struct Breadcrumbs {
    theme: Theme,
}

impl Breadcrumbs {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn show(&self, path: &[(&str, &str)]) {
        let mut row = String::new();
        for (i, (icon, label)) in path.iter().enumerate() {
            if i > 0 {
                row.push_str(&format!(" {} ", "›".with(self.theme.text_dim)));
            }
            row.push_str(&format!("{} {}", icon, label.with(self.theme.text)));
        }
        println!("  {}", row);
    }
}
