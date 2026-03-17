use crossterm::style::{Color, Stylize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Visual elements component for testing atomic UI pieces
#[derive(Debug, Clone)]
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

        // Use absolute path for the URL
        let abs_path = if std::path::Path::new(file_path).is_absolute() {
            file_path.to_string()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(file_path)
                .to_string_lossy()
                .to_string()
        };

        // Normalize path for URL: convert backslashes to forward slashes for URL format
        let normalized_path = abs_path.replace('\\', "/");

        // Use native URI schemes where possible for better IDE integration
        let url = match ide {
            Ide::VsCode => format!("vscode://file/{}", normalized_path),
            Ide::Zed => format!("zed://file/{}", normalized_path),
            Ide::ZedPreview => format!("zed-preview://file/{}", normalized_path),
        };

        // Fallback to file:// if the scheme isn't supported or doesn't work well
        // Many terminals and OS handlers work best with standard file:// URIs
        // For VSCode, it handles file:// URIs by default if it's the default handler.
        Self::new(text, &url)
    }

    /// Creates a generic file:// link that the OS will handle with the default application
    pub fn file_link(text: &str, file_path: &str) -> String {
        let abs_path = if std::path::Path::new(file_path).is_absolute() {
            file_path.to_string()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(file_path)
                .to_string_lossy()
                .to_string()
        };
        let normalized_path = abs_path.replace('\\', "/");
        let url = if cfg!(windows) {
            format!("file:///{}", normalized_path)
        } else {
            format!("file://{}", normalized_path)
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
