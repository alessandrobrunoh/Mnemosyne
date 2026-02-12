use crossterm::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub accent: Color,
    pub active: Color,
    pub secondary: Color,
    pub text_dim: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Rgb {
                r: 163,
                g: 133,
                b: 255,
            }, // #A385FF
            active: Color::Rgb {
                r: 109,
                g: 255,
                b: 216,
            }, // #6DFFD8
            secondary: Color::Rgb {
                r: 102,
                g: 102,
                b: 102,
            }, // #666666
            text_dim: Color::Rgb {
                r: 102,
                g: 102,
                b: 102,
            },
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            border: Color::DarkGrey,
        }
    }
}

pub const DEFAULT: Theme = Theme {
    accent: Color::Rgb {
        r: 163,
        g: 133,
        b: 255,
    },
    active: Color::Rgb {
        r: 109,
        g: 255,
        b: 216,
    },
    secondary: Color::Rgb {
        r: 102,
        g: 102,
        b: 102,
    },
    text_dim: Color::Rgb {
        r: 102,
        g: 102,
        b: 102,
    },
    success: Color::Green,
    error: Color::Red,
    warning: Color::Yellow,
    border: Color::DarkGrey,
};
