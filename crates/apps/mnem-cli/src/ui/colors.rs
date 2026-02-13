use crossterm::style::Color;

pub const PURPLE: Color = Color::Rgb {
    r: 189,
    g: 147,
    b: 249,
}; // #BD93F9
pub const PURPLE_BRIGHT: Color = Color::Rgb {
    r: 214,
    g: 183,
    b: 255,
}; // #D6B7FF
pub const CYAN: Color = Color::Rgb {
    r: 139,
    g: 233,
    b: 253,
}; // #8BE9FD
pub const CYAN_BRIGHT: Color = Color::Rgb {
    r: 172,
    g: 245,
    b: 255,
}; // #ACF5FF
pub const PINK: Color = Color::Rgb {
    r: 255,
    g: 121,
    b: 198,
}; // #FF79C6
pub const PINK_BRIGHT: Color = Color::Rgb {
    r: 255,
    g: 174,
    b: 224,
}; // #FFAEE0
pub const GREEN: Color = Color::Rgb {
    r: 80,
    g: 250,
    b: 123,
}; // #50FA7B
pub const GREEN_BRIGHT: Color = Color::Rgb {
    r: 121,
    g: 255,
    b: 163,
}; // #79FFA3
pub const RED: Color = Color::Rgb {
    r: 255,
    g: 85,
    b: 85,
}; // #FF5555
pub const RED_BRIGHT: Color = Color::Rgb {
    r: 255,
    g: 130,
    b: 130,
}; // #FF8282
pub const ORANGE: Color = Color::Rgb {
    r: 255,
    g: 184,
    b: 108,
}; // #FFB86C
pub const ORANGE_BRIGHT: Color = Color::Rgb {
    r: 255,
    g: 210,
    b: 140,
}; // #FFD28C
pub const YELLOW: Color = Color::Rgb {
    r: 241,
    g: 250,
    b: 140,
}; // #F1FA8C
pub const MINT: Color = Color::Rgb {
    r: 80,
    g: 250,
    b: 203,
}; // #50FACB
pub const COMMENT: Color = Color::Rgb {
    r: 98,
    g: 114,
    b: 164,
}; // #6272A8
pub const SELECTION: Color = Color::Rgb {
    r: 68,
    g: 71,
    b: 90,
}; // #44475A
pub const BG: Color = Color::Rgb {
    r: 40,
    g: 42,
    b: 54,
}; // #282A36
pub const BG_BRIGHT: Color = Color::Rgb {
    r: 50,
    g: 52,
    b: 66,
}; // #323442
pub const FG: Color = Color::Rgb {
    r: 248,
    g: 248,
    b: 242,
}; // #F8F8F2

pub const ACCENT: Color = PINK;
pub const ACTIVE: Color = CYAN;
pub const DIM: Color = COMMENT;
