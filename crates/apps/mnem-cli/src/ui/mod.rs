// UI Module for mnem-cli
//
// This module provides all UI-related functionality including:
// - Color definitions and utilities
// - Highlighting for code
// - Layout system with Builder Pattern
// - Reusable UI components (Header, List, Messages, Table, Status)
//
// # Example
// ```rust
// use mnem_cli::ui::{Layout, LayoutBuilder, Header, Messages};
//
// // Create layout
// let layout = LayoutBuilder::new().build();
//
// // Use components
// let header = Header::new(layout.theme());
// header.render("My Application");
// ```

// Core modules
pub mod colors;
pub mod highlight;
pub mod layout;

// Re-export highlighting
pub use highlight::TsHighlighter;

// Re-export colors
pub use colors::*;

// Re-export layout system
pub use layout::{Layout, LayoutBuilder};

// Re-export UI components and elements for convenience
pub use crate::ui_components::{Elements, Hyperlink, List, Messages, Status};
