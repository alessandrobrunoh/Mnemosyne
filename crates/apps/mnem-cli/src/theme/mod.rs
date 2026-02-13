//! Theme system for mnem-cli terminal UI
//!
//! This module provides a theming system with:
//! - Color palettes (currently Mnemosyne only)
//! - Theme configuration
//! - Builder pattern for custom themes
//!
//! # Example
//! ```rust
//! use mnem_cli::theme::{Theme, ThemeBuilder};
//!
//! // Get default theme
//! let theme = Theme::default();
//!
//! // Customize with builder
//! let custom = ThemeBuilder::new()
//!     .accent(crossterm::style::Color::Cyan)
//!     .build();
//! ```

// Module declarations
pub mod builder;
pub mod palette;
pub mod theme;

// Re-exports from palette module
pub use palette::{Palette, MNEMOSYNE};

// Re-exports from theme module
pub use theme::Theme;

// Re-exports from builder module
pub use builder::{get_theme, ThemeBuilder};

// Re-export helper functions from palette
pub use palette::{default as default_palette, from_name, list_available};
