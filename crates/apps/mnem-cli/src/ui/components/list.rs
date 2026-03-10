use crate::theme::Theme;
use crossterm::style::Stylize;

/// Pagination metadata for displaying list pagination information
///
/// Encapsulates all data needed to render pagination information consistently.
///
/// # Example
/// ```rust
/// use mnem_cli::ui::components::list::PaginationInfo;
///
/// let info = PaginationInfo::new(1, 100, 20)
///     .with_info("STATUS".to_string(), "Listing".to_string())
///     .with_info("FILE".to_string(), "Project".to_string());
/// ```
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub current_page: usize,
    pub total_items: usize,
    pub items_per_page: usize,
    pub additional_info: Vec<(String, String)>,
}

impl PaginationInfo {
    /// Create new pagination info
    ///
    /// # Arguments
    /// * `current_page` - Current page number (1-indexed)
    /// * `total_items` - Total number of items across all pages
    /// * `items_per_page` - Number of items displayed per page
    pub fn new(current_page: usize, total_items: usize, items_per_page: usize) -> Self {
        Self {
            current_page,
            total_items,
            items_per_page,
            additional_info: Vec::new(),
        }
    }

    /// Add additional metadata to display in pagination footer
    ///
    /// # Example
    /// ```rust
    /// let info = PaginationInfo::new(1, 100, 20)
    ///     .with_info("STATUS".to_string(), "Listing".to_string())
    ///     .with_info("FILE".to_string(), "Project".to_string());
    /// ```
    pub fn with_info(mut self, label: String, value: String) -> Self {
        self.additional_info.push((label, value));
        self
    }

    /// Calculate total number of pages
    pub fn total_pages(&self) -> usize {
        if self.total_items == 0 {
            1
        } else {
            (self.total_items as f64 / self.items_per_page as f64).ceil() as usize
        }
    }
}

/// List component for displaying formatted tabular data in terminal UI
///
/// Provides styled list rendering with theme-aware coloring and formatting.
/// Supports items, bullets, numbered entries, status indicators, and pagination.
///
/// # Example
/// ```rust
/// use mnem_cli::ui::components::list::List;
/// use mnem_cli::theme::Theme;
///
/// let list = List::new(Theme::default());
/// list.item("→", "First item");
/// list.bullet("Second item");
/// list.numbered(1, "Third item");
/// ```
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

    /// Display a bullet point item
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

    /// Display a numbered item
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

    /// Display an item with status indicator
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

    /// Display a nested item (indented)
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

    /// Display pagination information footer
    ///
    /// Shows current page, total pages, total items, and any additional metadata.
    /// Consistent formatting across all paginated views.
    ///
    /// # Example
    /// ```rust
    /// let info = PaginationInfo::new(1, 100, 20)
    ///     .with_info("STATUS".to_string(), "Listing".to_string())
    ///     .with_info("FILE".to_string(), "Project".to_string());
    /// list.pagination(&info);
    /// ```
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

    #[test]
    fn test_pagination_info() {
        let info = PaginationInfo::new(1, 100, 20);
        assert_eq!(info.current_page, 1);
        assert_eq!(info.total_items, 100);
        assert_eq!(info.items_per_page, 20);
        assert_eq!(info.total_pages(), 5);
    }

    #[test]
    fn test_pagination_info_with_additional() {
        let info = PaginationInfo::new(1, 100, 20)
            .with_info("STATUS".to_string(), "Listing".to_string())
            .with_info("FILE".to_string(), "Project".to_string());

        assert_eq!(info.additional_info.len(), 2);
        assert_eq!(info.additional_info[0].0, "STATUS");
        assert_eq!(info.additional_info[1].0, "FILE");
    }

    #[test]
    fn test_pagination_edge_cases() {
        // Zero items
        let info = PaginationInfo::new(1, 0, 20);
        assert_eq!(info.total_pages(), 1);

        // Exact page boundary
        let info = PaginationInfo::new(1, 100, 20);
        assert_eq!(info.total_pages(), 5);

        // Partial last page
        let info = PaginationInfo::new(1, 105, 20);
        assert_eq!(info.total_pages(), 6);
    }
}
