use crossterm::style::{Color, Stylize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Hyperlink;

impl Hyperlink {
    /// Creates an OSC 8 hyperlink.
    /// terminal support varies, but most modern ones support this.
    pub fn new(text: &str, url: &str) -> String {
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
    }

    /// Creates a custom uri for mnem actions (requires OS handler, but prepares the ground)
    pub fn action(text: &str, action: &str, payload: &str) -> String {
        let url = format!("mnem://{}/{}", action, payload);
        Self::new(text, &url)
    }

    /// Creates a direct link to open a file in a specific IDE using its native protocol
    pub fn ide_link(text: &str, file_path: &str, ide: &mnem_core::config::Ide) -> String {
        use mnem_core::config::Ide;
        let url = match ide {
            Ide::VsCode => format!("vscode://file{}", file_path),
            Ide::Zed => format!("zed://file{}", file_path),
            Ide::ZedPreview => format!("zed-preview://file{}", file_path),
        };
        Self::new(text, &url)
    }
}

pub struct BranchBadge;

impl BranchBadge {
    pub fn simple(branch_name: &str) -> String {
        let color = Self::color_from_string(branch_name);
        branch_name.with(color).bold().to_string()
    }

    fn color_from_string(s: &str) -> Color {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();

        // Simple mapping to a set of distinct colors
        let colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::DarkRed,
            Color::DarkGreen,
            Color::DarkYellow,
            Color::DarkBlue,
            Color::DarkMagenta,
            Color::DarkCyan,
        ];

        colors[(hash as usize) % colors.len()]
    }
}
