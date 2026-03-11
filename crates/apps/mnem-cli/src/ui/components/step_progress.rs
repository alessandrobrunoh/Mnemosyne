use crate::theme::Theme;
use crossterm::style::Stylize;

/// Multi-phase workflow status
#[derive(Debug, Clone)]
pub struct StepProgress {
    theme: Theme,
}

impl StepProgress {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render(&self, steps: &[(&str, bool, bool)]) {
        // (label, completed, is_current)
        let mut row = String::new();
        for (i, (label, completed, current)) in steps.iter().enumerate() {
            if i > 0 {
                row.push_str(&format!(" {} ", "──".with(self.theme.text_dim)));
            }

            let icon = if *completed {
                "(✓)".with(self.theme.success)
            } else if *current {
                "(●)".with(self.theme.active)
            } else {
                "(○)".with(self.theme.text_dim)
            };

            let text = if *current {
                label.with(self.theme.text_bright).bold()
            } else {
                label.with(self.theme.text_dim)
            };

            row.push_str(&format!("{} {}", icon, text));
        }
        println!("  {}", row);
    }
}
