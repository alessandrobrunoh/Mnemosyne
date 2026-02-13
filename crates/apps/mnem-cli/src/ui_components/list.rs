use crate::theme::Theme;
use crossterm::style::Stylize;
use mnem_macros::UiDebug;

/// List component for displaying formatted lists in terminal UI
///
/// Provides styled list rendering with theme-aware coloring and formatting.
///
/// # Example
/// ```rust
/// use mnem_cli::ui_components::List;
/// use mnem_cli::theme::Theme;
///
/// let list = List::new(Theme::default());
/// list.item("→", "First item");
/// list.bullet("Second item");
/// list.numbered(1, "Third item");
/// ```
#[derive(Debug, Clone, UiDebug)]
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
    }

    /// Create a new list component with given theme
    /// Create a new list component with given theme
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Display a single list item with a label
    ///
    /// # Example
    /// ```rust
    /// list.item("→", "Project initialized successfully");
    /// ```
    pub fn item(&self, label: &str, content: &str) {
        println!(
            "  {} {}",
            label.with(self.theme.accent).bold(),
            content.with(self.theme.text)
        );
    }

    /// Display a bullet point list item
    ///
    /// # Example
    /// ```rust
    /// list.bullet("File saved successfully");
    /// ```
    pub fn bullet(&self, content: &str) {
        println!(
            "  {} {}",
            "•".with(self.theme.accent),
            content.with(self.theme.text)
        );
    }

    /// Display a numbered list item
    ///
    /// # Example
    /// ```rust
    /// list.numbered(1, "First item");
    /// ```
    pub fn numbered(&self, number: usize, content: &str) {
        println!(
            "  {: <4} {}",
            format!("{}.", number).with(self.theme.accent).bold(),
            content.with(self.theme.text)
        );
    }

    /// Display a list item with status indicator
    ///
    /// # Example
    /// ```rust
    /// list.status_item("✓", "Task completed", Some("2.3s"));
    /// ```
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

    /// Display a nested list item (indented)
    ///
    /// # Example
    /// ```rust
    /// list.nested("Sub-item with additional information");
    /// ```
    pub fn nested(&self, content: &str) {
        println!(
            "    {} {}",
            "└─".with(self.theme.text_dim),
            content.with(self.theme.text_dim)
        );
    }

    /// Display multiple items from a slice
    ///
    /// # Example
    /// ```rust
    /// let items = vec![("First", "Description 1"), ("Second", "Description 2")];
    /// list.items(&items);
    /// ```
    pub fn items(&self, items: &[(impl AsRef<str>, impl AsRef<str>)]) {
        for (label, content) in items {
            self.item(label.as_ref(), content.as_ref());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_creation() {
        let theme = Theme::default();
        let list = List::new(theme);

        // Test that methods don't panic
        list.item("→", "Test item");
        list.bullet("Bullet item");
        list.numbered(1, "Numbered item");
        list.status_item("✓", "Status item", Some("1.2s"));
        list.nested("Nested item");
    }
}
