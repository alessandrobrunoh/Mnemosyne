use super::palette;
use crossterm::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    // Primary brand colors
    pub primary: Color,
    pub primary_bright: Color,
    pub secondary: Color,
    pub secondary_bright: Color,

    // Accent colors
    pub accent: Color,
    pub accent_soft: Color,
    pub accent_bright: Color,
    pub accent_alt: Color,

    // Semantic colors
    pub success: Color,
    pub success_bright: Color,
    pub success_soft: Color,
    pub error: Color,
    pub error_bright: Color,
    pub error_soft: Color,
    pub warning: Color,
    pub warning_bright: Color,
    pub warning_soft: Color,
    pub info: Color,
    pub info_bright: Color,
    pub info_soft: Color,

    // Text colors
    pub text: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    pub text_muted: Color,

    // Background colors
    pub background: Color,
    pub background_bright: Color,
    pub background_dim: Color,
    pub surface: Color,
    pub surface_elevated: Color,

    // Border colors
    pub border: Color,
    pub border_dim: Color,
    pub border_focus: Color,
    pub border_accent: Color,

    // Special elements (legacy alias)
    pub active: Color,
    pub highlight: Color,
    pub highlight_soft: Color,
    pub selection: Color,
    pub link: Color,
    pub link_visited: Color,

    // Code colors
    pub code_keyword: Color,
    pub code_string: Color,
    pub code_number: Color,
    pub code_comment: Color,
    pub code_function: Color,
    pub code_type: Color,

    // Diff colors
    pub diff_add: Color,
    pub diff_add_bg: Color,
    pub diff_remove: Color,
    pub diff_remove_bg: Color,
    pub diff_context: Color,
    pub diff_header: Color,

    // Timeline colors
    pub timeline_cyan: Color,
    pub timeline_pink: Color,
    pub timeline_orange: Color,
    pub timeline_yellow: Color,
    pub timeline_green: Color,
    pub timeline_purple: Color,
}

impl Default for Theme {
    fn default() -> Self {
        palette::MNEMOSYNE.to_theme()
    }
}

impl Theme {
    pub fn from_mnemosyne() -> Self {
        palette::MNEMOSYNE.to_theme()
    }

    pub fn from_palette(palette: palette::Palette) -> Self {
        palette.to_theme()
    }
}

impl palette::Palette {
    pub fn to_theme(&self) -> Theme {
        Theme {
            // Primary
            primary: self.primary,
            primary_bright: self.primary_bright,
            secondary: self.secondary,
            secondary_bright: self.secondary_bright,
            // Accent
            accent: self.accent,
            accent_soft: self.accent_soft,
            accent_bright: self.accent_bright,
            accent_alt: self.accent_alt,
            // Semantic
            success: self.success,
            success_bright: self.success_bright,
            success_soft: self.success_soft,
            error: self.error,
            error_bright: self.error_bright,
            error_soft: self.error_soft,
            warning: self.warning,
            warning_bright: self.warning_bright,
            warning_soft: self.warning_soft,
            info: self.info,
            info_bright: self.info_bright,
            info_soft: self.info_soft,
            // Text
            text: self.text,
            text_dim: self.text_dim,
            text_bright: self.text_bright,
            text_muted: self.text_muted,
            // Background
            background: self.background,
            background_bright: self.background_bright,
            background_dim: self.background_dim,
            surface: self.surface,
            surface_elevated: self.surface_elevated,
            // Border
            border: self.border,
            border_dim: self.border_dim,
            border_focus: self.border_focus,
            border_accent: self.border_accent,
            // Special
            active: self.primary,
            highlight: self.highlight,
            highlight_soft: self.highlight_soft,
            selection: self.selection,
            link: self.link,
            link_visited: self.link_visited,
            // Code
            code_keyword: self.code_keyword,
            code_string: self.code_string,
            code_number: self.code_number,
            code_comment: self.code_comment,
            code_function: self.code_function,
            code_type: self.code_type,
            // Diff
            diff_add: self.diff_add,
            diff_add_bg: self.diff_add_bg,
            diff_remove: self.diff_remove,
            diff_remove_bg: self.diff_remove_bg,
            diff_context: self.diff_context,
            diff_header: self.diff_header,
            // Timeline
            timeline_cyan: self.timeline_cyan,
            timeline_pink: self.timeline_pink,
            timeline_orange: self.timeline_orange,
            timeline_yellow: self.timeline_yellow,
            timeline_green: self.timeline_green,
            timeline_purple: self.timeline_purple,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.accent, palette::MNEMOSYNE.accent);
        assert_eq!(theme.success, palette::MNEMOSYNE.success);
        assert_eq!(theme.error, palette::MNEMOSYNE.error);
    }

    #[test]
    fn test_from_mnemosyne() {
        let theme = Theme::from_mnemosyne();
        assert_eq!(theme.accent, palette::MNEMOSYNE.accent);
        assert_eq!(theme.warning, palette::MNEMOSYNE.warning);
    }

    #[test]
    fn test_palette_to_theme() {
        let palette = palette::MNEMOSYNE;
        let theme = palette.to_theme();

        assert_eq!(theme.accent, palette.accent);
        assert_eq!(theme.secondary, palette.secondary);
        assert_eq!(theme.success, palette.success);
        assert_eq!(theme.error, palette.error);
        assert_eq!(theme.warning, palette.warning);
        assert_eq!(theme.border, palette.border);
    }
}
