use crossterm::style::{Color, Stylize};
use mnem_macros::UiDebug;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Visual elements component for testing atomic UI pieces
#[derive(Debug, Clone, UiDebug)]
pub struct Elements {
    theme: crate::theme::Theme,
}

impl Elements {
    pub fn new(theme: crate::theme::Theme) -> Self {
        Self { theme }
    }

    pub fn test_output(&self) {
        println!("HYPERLINKS:");
        println!(
            "  Standard: {}",
            Hyperlink::new("Google", "https://google.com")
        );
        println!(
            "  Action:   {}",
            Hyperlink::action("Open Vault", "open", "vault_123")
        );
        println!();

        println!("BRANCH BADGES:");
        println!("  Main:   {}", BranchBadge::simple("main"));
        println!("  Dev:    {}", BranchBadge::simple("feature/ui-refactor"));
        println!("  Fix:    {}", BranchBadge::simple("fix/bug-42"));
    }
}

/// Utility for creating terminal hyperlinks (OSC 8)
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
    /// Works cross-platform: Windows, Linux, macOS
    pub fn ide_link(text: &str, file_path: &str, ide: &mnem_core::config::Ide) -> String {
        use mnem_core::config::Ide;

        // Normalize path for URL: convert backslashes to forward slashes
        let normalized_path = file_path.replace('\\', "/");

        // Use file:// which opens in default application or the system default
        // This works better cross-platform than custom URI schemes
        let url = match ide {
            Ide::VsCode => format!("vscode://file/{}", normalized_path),
            Ide::Zed => format!("file:///{}", normalized_path),
            Ide::ZedPreview => format!("file:///{}", normalized_path),
        };
        Self::new(text, &url)
    }
}

/// Visual badge for Git branches with consistent coloring
pub struct BranchBadge;

impl BranchBadge {
    /// Create a styled branch name with a color derived from its name
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
