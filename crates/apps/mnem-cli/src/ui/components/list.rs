use crate::theme::Theme;
use crate::ui::components::pagination::PaginationInfo;
use crossterm::style::Stylize;

/// List component for displaying formatted tabular data in terminal UI
///
/// Provides styled list rendering with theme-aware coloring and formatting.
/// Supports items, bullets, numbered entries, status indicators, and pagination.
#[derive(Debug, Clone)]
pub struct List {
    theme: Theme,
}

impl List {
    /// Internal test rendering logic
    pub fn test_output(&self) {
        self.item("1", "First item");
        self.bullet("Bullet item");
        self.numbered(3, "Numbered item");
        self.status_item("✓", "Success item", Some("v1.0"));
        let info =
            PaginationInfo::new(1, 100, 20).with_info("STATUS".to_string(), "Listing".to_string());
        self.pagination(&info);
    }

    /// Create a new list component with given theme
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Display a single list row item with a label
    pub fn item(&self, label: &str, content: &str) {
        println!(
            "  {} {}",
            label.with(self.theme.accent).bold(),
            content.with(self.theme.text)
        );
    }

    /// Display a bullet point item
    pub fn bullet(&self, content: &str) {
        println!(
            "  {} {}",
            "•".with(self.theme.accent),
            content.with(self.theme.text)
        );
    }

    /// Display a numbered item
    pub fn numbered(&self, number: usize, content: &str) {
        println!(
            "  {: <4} {}",
            format!("{}.", number).with(self.theme.accent).bold(),
            content.with(self.theme.text)
        );
    }

    /// Display an item with status indicator
    pub fn status_item(&self, status: &str, content: &str, meta: Option<&str>) {
        let meta_str = match meta {
            Some(m) => format!(" {}", m.with(self.theme.text_dim)),
            None => String::new(),
        };
        println!(
            "  {} {}{}",
            status.with(self.theme.success).bold(),
            content.with(self.theme.text),
            meta_str
        );
    }

    /// Display a nested item (indented)
    pub fn nested(&self, content: &str) {
        println!(
            "    {} {}",
            "└─".with(self.theme.text_dim),
            content.with(self.theme.text_dim)
        );
    }

    /// Display multiple items from a slice
    pub fn items(&self, items: &[(impl AsRef<str>, impl AsRef<str>)]) {
        for (label, content) in items {
            self.item(label.as_ref(), content.as_ref());
        }
    }

    /// Display pagination information footer
    pub fn pagination(&self, info: &PaginationInfo) {
        let total_pages = info.total_pages();

        // Build the output string with consistent spacing
        let mut parts = vec![
            format!(
                "{} {}/{}",
                "PAGE".with(self.theme.text_dim).bold(),
                info.current_page.to_string().with(self.theme.text_bright),
                total_pages.to_string().with(self.theme.text_dim)
            ),
            format!(
                "{} {}",
                "TOTAL".with(self.theme.text_dim).bold(),
                info.total_items.to_string().with(self.theme.text_bright)
            ),
        ];

        // Add additional info (STATUS, FILE, etc.)
        for (label, value) in &info.additional_info {
            parts.push(format!(
                "{} {}",
                label.as_str().with(self.theme.text_dim).bold(),
                value.as_str().with(self.theme.text_bright)
            ));
        }

        println!();
        println!("  {}", parts.join("  "));
    }
}
