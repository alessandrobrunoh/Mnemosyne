use crate::theme::Theme;
use crossterm::style::Stylize;

/// Pagination metadata for displaying pagination information consistently across components
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub current_page: usize,
    pub total_items: usize,
    pub items_per_page: usize,
    pub additional_info: Vec<(String, String)>,
}

impl PaginationInfo {
    /// Create new pagination info
    pub fn new(current_page: usize, total_items: usize, items_per_page: usize) -> Self {
        Self {
            current_page,
            total_items,
            items_per_page,
            additional_info: Vec::new(),
        }
    }

    /// Add additional metadata to display in pagination footer
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
