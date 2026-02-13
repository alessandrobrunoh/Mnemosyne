use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    // Primary colors - brand identity
    pub primary: Color,
    pub primary_bright: Color,
    pub secondary: Color,
    pub secondary_bright: Color,

    // Accent colors - highlights and interactive elements
    pub accent: Color,
    pub accent_soft: Color,
    pub accent_bright: Color,
    pub accent_alt: Color,

    // Semantic colors - status and meaning
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

    // Background colors - depth layers
    pub background: Color,
    pub background_bright: Color,
    pub background_dim: Color,
    pub surface: Color,
    pub surface_elevated: Color,

    // Foreground/text colors
    pub foreground: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    pub text_muted: Color,

    // Border colors - structure and separation
    pub border: Color,
    pub border_dim: Color,
    pub border_focus: Color,
    pub border_accent: Color,

    // Special UI elements
    pub active: Color,
    pub highlight: Color,
    pub highlight_soft: Color,
    pub selection: Color,
    pub link: Color,
    pub link_visited: Color,

    // Code/Syntax colors
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

    // Timeline/History colors
    pub timeline_cyan: Color,
    pub timeline_pink: Color,
    pub timeline_orange: Color,
    pub timeline_yellow: Color,
    pub timeline_green: Color,
    pub timeline_purple: Color,
}

pub const MNEMOSYNE: Palette = Palette {
    // Primary colors - Vibrant Cyan
    primary: Color::Rgb {
        r: 139,
        g: 233,
        b: 253,
    },
    primary_bright: Color::Rgb {
        r: 170,
        g: 245,
        b: 255,
    },
    secondary: Color::Rgb {
        r: 189,
        g: 147,
        b: 249,
    },
    secondary_bright: Color::Rgb {
        r: 210,
        g: 180,
        b: 255,
    },

    // Accent colors - Purple/Pink
    accent: Color::Rgb {
        r: 255,
        g: 121,
        b: 198,
    },
    accent_soft: Color::Rgb {
        r: 255,
        g: 150,
        b: 220,
    },
    accent_bright: Color::Rgb {
        r: 255,
        g: 180,
        b: 255,
    },
    accent_alt: Color::Rgb {
        r: 189,
        g: 147,
        b: 249,
    },

    // Semantic colors
    success: Color::Rgb {
        r: 80,
        g: 250,
        b: 123,
    },
    success_bright: Color::Rgb {
        r: 120,
        g: 255,
        b: 160,
    },
    success_soft: Color::Rgb {
        r: 60,
        g: 200,
        b: 100,
    },
    error: Color::Rgb {
        r: 255,
        g: 85,
        b: 85,
    },
    error_bright: Color::Rgb {
        r: 255,
        g: 120,
        b: 120,
    },
    error_soft: Color::Rgb {
        r: 200,
        g: 60,
        b: 60,
    },
    warning: Color::Rgb {
        r: 255,
        g: 184,
        b: 108,
    },
    warning_bright: Color::Rgb {
        r: 255,
        g: 210,
        b: 150,
    },
    warning_soft: Color::Rgb {
        r: 200,
        g: 140,
        b: 80,
    },
    info: Color::Rgb {
        r: 139,
        g: 233,
        b: 253,
    },
    info_bright: Color::Rgb {
        r: 170,
        g: 245,
        b: 255,
    },
    info_soft: Color::Rgb {
        r: 100,
        g: 180,
        b: 200,
    },

    // Background colors
    background: Color::Rgb {
        r: 40,
        g: 42,
        b: 54,
    },
    background_bright: Color::Rgb {
        r: 68,
        g: 71,
        b: 90,
    },
    background_dim: Color::Rgb {
        r: 30,
        g: 30,
        b: 40,
    },
    surface: Color::Rgb {
        r: 50,
        g: 50,
        b: 70,
    },
    surface_elevated: Color::Rgb {
        r: 70,
        g: 70,
        b: 95,
    },

    // Foreground/text colors
    foreground: Color::Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    text: Color::Rgb {
        r: 248,
        g: 248,
        b: 242,
    },
    text_dim: Color::Rgb {
        r: 144,
        g: 144,
        b: 144,
    },
    text_bright: Color::Rgb {
        r: 255,
        g: 255,
        b: 255,
    },
    text_muted: Color::Rgb {
        r: 98,
        g: 114,
        b: 164,
    },

    // Border colors
    border: Color::Rgb {
        r: 68,
        g: 71,
        b: 90,
    },
    border_dim: Color::Rgb {
        r: 50,
        g: 50,
        b: 70,
    },
    border_focus: Color::Rgb {
        r: 189,
        g: 147,
        b: 249,
    },
    border_accent: Color::Rgb {
        r: 255,
        g: 121,
        b: 198,
    },

    // Special UI elements
    active: Color::Rgb {
        r: 80,
        g: 250,
        b: 123,
    },
    highlight: Color::Rgb {
        r: 241,
        g: 250,
        b: 140,
    },
    highlight_soft: Color::Rgb {
        r: 200,
        g: 200,
        b: 100,
    },
    selection: Color::Rgb {
        r: 68,
        g: 71,
        b: 90,
    },
    link: Color::Rgb {
        r: 139,
        g: 233,
        b: 253,
    },
    link_visited: Color::Rgb {
        r: 189,
        g: 147,
        b: 249,
    },

    // Code/Syntax colors
    code_keyword: Color::Rgb {
        r: 255,
        g: 121,
        b: 198,
    },
    code_string: Color::Rgb {
        r: 241,
        g: 250,
        b: 140,
    },
    code_number: Color::Rgb {
        r: 189,
        g: 147,
        b: 249,
    },
    code_comment: Color::Rgb {
        r: 98,
        g: 114,
        b: 164,
    },
    code_function: Color::Rgb {
        r: 80,
        g: 250,
        b: 123,
    },
    code_type: Color::Rgb {
        r: 139,
        g: 233,
        b: 253,
    },

    // Diff colors
    diff_add: Color::Rgb {
        r: 80,
        g: 250,
        b: 123,
    },
    diff_add_bg: Color::Rgb {
        r: 50,
        g: 70,
        b: 50,
    },
    diff_remove: Color::Rgb {
        r: 255,
        g: 85,
        b: 85,
    },
    diff_remove_bg: Color::Rgb {
        r: 70,
        g: 50,
        b: 50,
    },
    diff_context: Color::Rgb {
        r: 98,
        g: 114,
        b: 164,
    },
    diff_header: Color::Rgb {
        r: 248,
        g: 248,
        b: 242,
    },

    // Timeline/History colors
    timeline_cyan: Color::Rgb {
        r: 139,
        g: 233,
        b: 253,
    },
    timeline_pink: Color::Rgb {
        r: 255,
        g: 121,
        b: 198,
    },
    timeline_orange: Color::Rgb {
        r: 255,
        g: 184,
        b: 108,
    },
    timeline_yellow: Color::Rgb {
        r: 241,
        g: 250,
        b: 140,
    },
    timeline_green: Color::Rgb {
        r: 80,
        g: 250,
        b: 123,
    },
    timeline_purple: Color::Rgb {
        r: 189,
        g: 147,
        b: 249,
    },
};

pub const fn default() -> Palette {
    MNEMOSYNE
}

pub fn from_name(name: &str) -> Option<Palette> {
    match name.to_lowercase().as_str() {
        "mnemosyne" | "default" | "" => Some(MNEMOSYNE),
        _ => None,
    }
}

pub fn list_available() -> &'static [&'static str] {
    &["default", "mnemosyne"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_palette() {
        let p = default();
        assert_eq!(p, MNEMOSYNE);
    }

    #[test]
    fn test_from_name() {
        assert!(from_name("mnemosyne").is_some());
        assert!(from_name("MNEMOSYNE").is_some());
        assert!(from_name("default").is_some());
        assert!(from_name("").is_some());
        assert!(from_name("unknown").is_none());
    }

    #[test]
    fn test_list_available() {
        let names = list_available();
        assert!(names.contains(&"default"));
        assert!(names.contains(&"mnemosyne"));
    }
}
