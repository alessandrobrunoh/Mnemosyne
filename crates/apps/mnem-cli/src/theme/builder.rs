use super::theme::Theme;
use crossterm::style::Color;

/// Builder for creating customized Theme instances
///
/// # Example
/// ```rust
/// use mnem_cli::theme::ThemeBuilder;
/// use crossterm::style::Color;
///
/// // Customize specific colors
/// let theme = ThemeBuilder::new()
///     .accent(Color::Cyan)
///     .error(Color::Red)
///     .build();
///
/// // Build completely from scratch
/// let custom = ThemeBuilder::new()
///     .accent(Color::Rgb { r: 255, g: 100, b: 150 })
///     .active(Color::Blue)
///     .success(Color::Green)
///     .error(Color::Red)
///     .warning(Color::Yellow)
///     .border(Color::Grey)
///     .text_dim(Color::DarkGrey)
///     .secondary(Color::Rgb { r: 100, g: 100, b: 100 })
///     .build();
/// ```
pub struct ThemeBuilder {
    theme: Theme,
}

impl Default for ThemeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeBuilder {
    /// Create a new builder with Mnemosyne colors
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
        }
    }

    /// Set accent color (highlights, interactive elements)
    pub fn accent(&mut self, color: Color) -> &mut Self {
        self.theme.accent = color;
        self
    }

    /// Set active/primary color (selected items)
    pub fn active(&mut self, color: Color) -> &mut Self {
        self.theme.active = color;
        self
    }

    /// Set secondary color (supporting elements)
    pub fn secondary(&mut self, color: Color) -> &mut Self {
        self.theme.secondary = color;
        self
    }

    /// Set dimmed text color
    pub fn text_dim(&mut self, color: Color) -> &mut Self {
        self.theme.text_dim = color;
        self
    }

    /// Set success indication color
    pub fn success(&mut self, color: Color) -> &mut Self {
        self.theme.success = color;
        self
    }

    /// Set error indication color
    pub fn error(&mut self, color: Color) -> &mut Self {
        self.theme.error = color;
        self
    }

    /// Set warning indication color
    pub fn warning(&mut self, color: Color) -> &mut Self {
        self.theme.warning = color;
        self
    }

    /// Set border color
    pub fn border(&mut self, color: Color) -> &mut Self {
        self.theme.border = color;
        self
    }

    /// Build final Theme instance
    ///
    /// This consumes builder and returns configured Theme.
    pub fn build(&self) -> Theme {
        self.theme.clone()
    }
}

/// Get default Mnemosyne theme
///
/// # Example
/// ```rust
/// use mnem_cli::theme::builder::get_theme;
///
/// let theme = get_theme();
/// ```
pub fn get_theme() -> Theme {
    Theme::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let theme = ThemeBuilder::new()
            .accent(Color::Cyan)
            .error(Color::Red)
            .build();

        assert_eq!(theme.accent, Color::Cyan);
        assert_eq!(theme.error, Color::Red);
        // Other colors should be from Mnemosyne palette
        assert_eq!(theme.success, super::super::palette::MNEMOSYNE.success);
    }

    #[test]
    fn test_builder_complete_custom() {
        let theme = ThemeBuilder::new()
            .accent(Color::Magenta)
            .active(Color::Blue)
            .success(Color::Green)
            .error(Color::Red)
            .warning(Color::Yellow)
            .border(Color::Grey)
            .text_dim(Color::DarkGrey)
            .secondary(Color::Rgb {
                r: 100,
                g: 100,
                b: 100,
            })
            .build();

        assert_eq!(theme.accent, Color::Magenta);
        assert_eq!(theme.active, Color::Blue);
        assert_eq!(theme.success, Color::Green);
        assert_eq!(theme.error, Color::Red);
        assert_eq!(theme.warning, Color::Yellow);
        assert_eq!(theme.border, Color::Grey);
        assert_eq!(theme.text_dim, Color::DarkGrey);
    }

    #[test]
    fn test_get_theme() {
        let theme = get_theme();
        assert_eq!(theme.accent, super::super::palette::MNEMOSYNE.accent);
    }

    #[test]
    fn test_builder_default() {
        let theme = ThemeBuilder::new().build();
        let default = Theme::default();
        assert_eq!(theme.accent, default.accent);
        assert_eq!(theme.error, default.error);
    }
}
