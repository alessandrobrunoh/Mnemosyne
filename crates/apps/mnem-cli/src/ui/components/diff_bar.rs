use crate::theme::Theme;
use crossterm::style::Stylize;

/// A compact visual summary of changes
#[derive(Debug, Clone)]
pub struct DiffBar {
    theme: Theme,
    width: usize,
}

impl DiffBar {
    pub fn new(theme: Theme) -> Self {
        Self { theme, width: 10 }
    }

    pub fn render(&self, added: usize, removed: usize) {
        let total = added + removed;
        if total == 0 {
            println!("  [{: <width$}]  +0 / -0", "", width = self.width);
            return;
        }

        let add_chars = (added as f64 / total as f64 * self.width as f64).round() as usize;
        let rem_chars = self.width - add_chars;

        let bar_add = "+".repeat(add_chars).with(self.theme.success);
        let bar_rem = "-".repeat(rem_chars).with(self.theme.error);

        println!(
            "  [{}{}]  {} / {}",
            bar_add,
            bar_rem,
            format!("+{}", added).with(self.theme.success),
            format!("-{}", removed).with(self.theme.error)
        );
    }
}
