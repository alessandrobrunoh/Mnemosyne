use crate::theme::Theme;
use crate::ui::components::pagination::PaginationInfo;
use crossterm::style::Stylize;

/// Column configuration for the Smart Table
#[derive(Debug, Clone)]
pub struct Column {
    pub header: String,
    pub width: usize,
    pub flex: bool,
}

impl Column {
    pub fn new(header: &str, width: usize) -> Self {
        Self {
            header: header.to_string(),
            width,
            flex: false,
        }
    }

    pub fn flex(header: &str, min_width: usize) -> Self {
        Self {
            header: header.to_string(),
            width: min_width,
            flex: true,
        }
    }
}

/// A Smart & Symbolic Table component
#[derive(Debug, Clone)]
pub struct Table {
    theme: Theme,
    columns: Vec<Column>,
}

impl Table {
    pub fn new(theme: Theme, columns: Vec<Column>) -> Self {
        Self { theme, columns }
    }

    /// Render the table header with a thin separator
    pub fn header(&self) {
        let mut header_row = String::new();
        let mut separator_row = String::new();

        for (i, col) in self.columns.iter().enumerate() {
            let space = if i == 0 { "" } else { "  " };
            header_row.push_str(space);
            separator_row.push_str(space);

            let header = format!("{: <width$}", col.header, width = col.width);
            header_row.push_str(&header.with(self.theme.text_dim).bold().to_string());

            let sep = "─".repeat(col.width);
            separator_row.push_str(&sep.with(self.theme.text_dim).to_string());
        }

        println!("  {}", header_row);
        println!("  {}", separator_row);
    }

    /// Render a single data row
    pub fn row(&self, data: &[String]) {
        let mut row_str = String::new();

        for (i, col) in self.columns.iter().enumerate() {
            let space = if i == 0 { "" } else { "  " };
            row_str.push_str(space);

            let val = data.get(i).cloned().unwrap_or_default();
            let truncated = if val.len() > col.width {
                format!("{}…", &val[..col.width - 1])
            } else {
                format!("{: <width$}", val, width = col.width)
            };

            // Highlight based on column index or content could be added here
            if i == 0 {
                // Type/Icon column
                row_str.push_str(&truncated.with(self.theme.accent).bold().to_string());
            } else if i == 1 {
                // Hash column
                row_str.push_str(&truncated.with(self.theme.text_bright).to_string());
            } else {
                row_str.push_str(&truncated.with(self.theme.text).to_string());
            }
        }

        println!("  {}", row_str);
    }

    /// Display pagination information footer
    pub fn pagination(&self, info: &PaginationInfo) {
        let total_pages = info.total_pages();

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
