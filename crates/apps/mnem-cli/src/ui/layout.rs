use super::colors::{ACCENT, ACTIVE, DIM};
use crossterm::style::{Color, Stylize};

pub struct ButlerLayout;

impl ButlerLayout {
    pub fn header(title: &str) {
        println!("\n  {}", title.bold().cyan());
        println!("  {}", "━".repeat(title.len() + 2).dark_grey());
    }

    pub fn section_start(label: &str, title: &str) {
        println!("┊");
        println!(
            "┊{} {} [{}]",
            "╭┄".with(DIM),
            label.with(ACCENT).bold(),
            title.with(Color::White).bold()
        );
    }

    pub fn row_snapshot(hash_display: &str, content: &str) {
        println!("┊{}   {: <8} {}", "●".with(ACCENT), hash_display, content);
    }

    pub fn row_snapshot_latest(hash_display: &str, content: &str) {
        println!("┊{}   {: <8} {}", "●".with(ACTIVE), hash_display, content);
    }

    pub fn row_list(index: &str, content: &str) {
        println!("┊{: <12} {}", index.with(DIM), content);
    }

    pub fn item_simple(content: &str) {
        println!("┊   {}", content);
    }

    pub fn legend(items: &[(&str, &str)]) {
        let mut legend = String::new();
        for (icon, desc) in items {
            legend.push_str(&format!("{} {}   ", icon, desc.dark_grey()));
        }
        println!("┊   {}", legend);
    }

    pub fn section_end() {
        println!("├╯");
    }

    pub fn footer(hint: &str) {
        println!("┊");
        println!("  {} {}", "💡".yellow(), hint.with(DIM).italic());
        println!("");
    }
}
