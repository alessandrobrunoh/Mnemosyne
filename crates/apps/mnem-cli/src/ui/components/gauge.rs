use crate::theme::Theme;
use crossterm::style::Stylize;

/// Resource status or progress bar
#[derive(Debug, Clone)]
pub struct Gauge {
    theme: Theme,
    width: usize,
}

impl Gauge {
    pub fn new(theme: Theme) -> Self {
        Self { theme, width: 20 }
    }

    pub fn render(&self, label: &str, percentage: f64) {
        let filled = (percentage * self.width as f64).round() as usize;
        let empty = self.width - filled;

        let bar_filled = "█".repeat(filled).with(if percentage > 0.9 {
            self.theme.error
        } else if percentage > 0.7 {
            self.theme.warning
        } else {
            self.theme.success
        });

        let bar_empty = "░".repeat(empty).with(self.theme.text_dim);

        println!(
            "  {: <15} [{}{}] {:.0}%",
            label.with(self.theme.text),
            bar_filled,
            bar_empty,
            percentage * 100.0
        );
    }
}
