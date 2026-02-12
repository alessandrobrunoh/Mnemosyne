use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub sidebar: Color,
    pub surface: Color,
    pub accent: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub border: Color,
    pub success: Color,
    pub diff_add_bg: Color,
    pub diff_add_fg: Color,
    pub diff_del_bg: Color,
    pub diff_del_fg: Color,
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Mnemosyne",
        bg: Color::Rgb(15, 17, 18),      // #0F1112
        sidebar: Color::Rgb(30, 33, 34), // #1E2122
        surface: Color::Rgb(30, 33, 34),
        accent: Color::Rgb(163, 133, 255),    // #A385FF
        text_main: Color::Rgb(109, 255, 216), // #6DFFD8
        text_dim: Color::Rgb(102, 102, 102),  // #666666
        border: Color::Rgb(30, 33, 34),       // #1E2122
        success: Color::Rgb(109, 255, 216),
        diff_add_bg: Color::Rgb(20, 40, 35),
        diff_add_fg: Color::Rgb(109, 255, 216),
        diff_del_bg: Color::Rgb(50, 20, 20),
        diff_del_fg: Color::Rgb(255, 109, 148),
    },
    Theme {
        name: "GitButler",
        bg: Color::Rgb(15, 15, 17),      // Deeper black
        sidebar: Color::Rgb(22, 22, 25), // Slightly lighter but still deep
        surface: Color::Rgb(32, 32, 35),
        accent: Color::Rgb(137, 180, 250), // Brighter Blue (Lavender/Blue)
        text_main: Color::Rgb(245, 245, 250), // Almost Pure White
        text_dim: Color::Rgb(160, 160, 170), // Brighter Grey
        border: Color::Rgb(45, 45, 50),
        success: Color::Rgb(166, 227, 161), // Brighter Pastel Green
        diff_add_bg: Color::Rgb(30, 50, 40),
        diff_add_fg: Color::Rgb(166, 227, 161),
        diff_del_bg: Color::Rgb(70, 30, 30),
        diff_del_fg: Color::Rgb(243, 139, 168), // Brighter Red
    },
    Theme {
        name: "One Dark",
        bg: Color::Rgb(40, 44, 52),
        sidebar: Color::Rgb(33, 37, 43),
        surface: Color::Rgb(44, 50, 60),
        accent: Color::Rgb(97, 175, 239),
        text_main: Color::Rgb(171, 178, 191),
        text_dim: Color::Rgb(92, 99, 112),
        border: Color::Rgb(56, 63, 75),
        success: Color::Rgb(152, 195, 121),
        diff_add_bg: Color::Rgb(43, 49, 43),
        diff_add_fg: Color::Rgb(152, 195, 121),
        diff_del_bg: Color::Rgb(55, 43, 43),
        diff_del_fg: Color::Rgb(224, 108, 117),
    },
    Theme {
        name: "Zed Dark",
        bg: Color::Rgb(24, 24, 27),
        sidebar: Color::Rgb(28, 28, 30),
        surface: Color::Rgb(35, 35, 38),
        accent: Color::Rgb(82, 145, 226),
        text_main: Color::Rgb(212, 212, 216),
        text_dim: Color::Rgb(113, 113, 122),
        border: Color::Rgb(40, 40, 45),
        success: Color::Rgb(34, 197, 94),
        diff_add_bg: Color::Rgb(25, 40, 32),
        diff_add_fg: Color::Rgb(74, 222, 128),
        diff_del_bg: Color::Rgb(45, 25, 25),
        diff_del_fg: Color::Rgb(248, 113, 113),
    },
    Theme {
        name: "Darcula",
        bg: Color::Rgb(43, 43, 43),
        sidebar: Color::Rgb(60, 63, 65),
        surface: Color::Rgb(60, 63, 65),
        accent: Color::Rgb(175, 177, 179),
        text_main: Color::Rgb(169, 183, 198),
        text_dim: Color::Rgb(96, 107, 118),
        border: Color::Rgb(81, 81, 81),
        success: Color::Rgb(98, 151, 85),
        diff_add_bg: Color::Rgb(41, 56, 41),
        diff_add_fg: Color::Rgb(98, 151, 85),
        diff_del_bg: Color::Rgb(70, 40, 40),
        diff_del_fg: Color::Rgb(188, 82, 82),
    },
    Theme {
        name: "JetBrains Dark",
        bg: Color::Rgb(30, 31, 34),
        sidebar: Color::Rgb(43, 45, 48),
        surface: Color::Rgb(43, 45, 48),
        accent: Color::Rgb(53, 116, 240),
        text_main: Color::Rgb(223, 225, 229),
        text_dim: Color::Rgb(110, 115, 125),
        border: Color::Rgb(62, 64, 70),
        success: Color::Rgb(89, 168, 105),
        diff_add_bg: Color::Rgb(38, 51, 41),
        diff_add_fg: Color::Rgb(89, 168, 105),
        diff_del_bg: Color::Rgb(60, 38, 41),
        diff_del_fg: Color::Rgb(190, 80, 90),
    },
    Theme {
        name: "Dracula",
        bg: Color::Rgb(40, 42, 54),
        sidebar: Color::Rgb(68, 71, 90),
        surface: Color::Rgb(68, 71, 90),
        accent: Color::Rgb(189, 147, 249),
        text_main: Color::Rgb(248, 248, 242),
        text_dim: Color::Rgb(98, 114, 164),
        border: Color::Rgb(98, 114, 164),
        success: Color::Rgb(80, 250, 123),
        diff_add_bg: Color::Rgb(50, 80, 50),
        diff_add_fg: Color::Rgb(80, 250, 123),
        diff_del_bg: Color::Rgb(80, 40, 40),
        diff_del_fg: Color::Rgb(255, 85, 85),
    },
];
