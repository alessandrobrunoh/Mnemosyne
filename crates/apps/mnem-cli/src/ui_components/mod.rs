// UI Components Module
//
// This module provides reusable UI components that integrate with the theme system.
// Each component is self-contained and theme-aware.

pub mod elements;
pub mod list;
pub mod messages;
pub mod status;

/// Trait for UI components that can be tested and debugged
pub trait UIComponent: std::fmt::Debug {
    /// Return the machine-readable name of the component
    fn name(&self) -> &str;

    /// Render a comprehensive test suite for this component
    fn render_test(&self);
}

// Re-export public APIs

pub use elements::{Elements, Hyperlink};
pub use list::List;
pub use messages::Messages;
pub use status::Status;
