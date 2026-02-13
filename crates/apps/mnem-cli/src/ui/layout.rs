use crate::theme::Theme;
use crate::ui_components::elements::Hyperlink;
use crossterm::style::Stylize;
use mnem_macros::UiDebug;

pub struct LayoutBuilder {
    theme: Option<Theme>,
    padding: usize,
    separator: String,
    show_borders: bool,
    compact_mode: bool,
}

impl Default for LayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutBuilder {
    pub fn new() -> Self {
        Self {
            theme: None,
            padding: 2,
            separator: "─".to_string(),
            show_borders: true,
            compact_mode: false,
        }
    }

    pub fn theme(&mut self, theme: Theme) -> &mut Self {
        self.theme = Some(theme);
        self
    }

    pub fn padding(&mut self, padding: usize) -> &mut Self {
        self.padding = padding;
        self
    }

    pub fn separator(&mut self, separator: &str) -> &mut Self {
        self.separator = separator.to_string();
        self
    }

    pub fn show_borders(&mut self, show: bool) -> &mut Self {
        self.show_borders = show;
        self
    }

    pub fn compact(&mut self, compact: bool) -> &mut Self {
        self.compact_mode = compact;
        self
    }

    pub fn build(&self) -> Layout {
        Layout {
            theme: self.theme.clone().unwrap_or_else(Theme::default),
            padding: self.padding,
            separator: self.separator.clone(),
            show_borders: self.show_borders,
            compact_mode: self.compact_mode,
        }
    }
}

#[derive(Debug, Clone, UiDebug)]
pub struct Layout {
    theme: Theme,
    padding: usize,
    separator: String,
    show_borders: bool,
    compact_mode: bool,
}

impl Layout {
    pub fn test_output(&self) {
        self.header("TEST LAYOUT ENGINE");
        self.section_start("TEST", "Main Section");
        self.row_property("Engine", "Butler v2.0");
        self.row_property("Status", "Operational");
        self.row_snapshot("abcdef12", "main.rs");
        self.row_snapshot_latest("7890abcd", "lib.rs");
        self.row_list("1", "First item in list");
        self.item_simple("A simple line of text with padding");
        self.legend(&[("●", "Active"), ("•", "Inactive"), ("✓", "Synced")]);
        self.section_end();
        self.usage("test", "<param> [--flag]");
        self.footer("Layout engine testing complete");
    }

    pub fn new() -> Self {
        LayoutBuilder::new().build()
    }

    pub fn builder() -> LayoutBuilder {
        LayoutBuilder::new()
    }

    pub fn header_dashboard(&self, title: &str) {
        println!();
        println!(
            "  {}",
            title
                .to_uppercase()
                .with(self.theme.primary)
                .bold()
                .underlined()
        );
        println!(); // Spacer instead of bar for cleaner look with tracking
    }

    pub fn section_timeline(&self, code: &str, title: &str) {
        println!();
        println!(
            "  {} {}",
            "◆".with(self.theme.accent),
            title.with(self.theme.text_bright).bold()
        );
        println!("  {}", "─".repeat(40).with(self.theme.border_dim));
    }

    pub fn section_end(&self) {
        println!();
    }

    pub fn row_timeline(&self, icon: &str, content: &str) {
        println!("    {} {}", icon, content);
    }

    pub fn row_history(&self, hash: &str, time: &str, msg: &str, is_latest: bool) {
        let icon = if is_latest {
            "●".with(self.theme.success)
        } else {
            " ".with(self.theme.text_muted) // Cleaner look
        };
        let hash_styled = if is_latest {
            hash.with(self.theme.success).bold()
        } else {
            hash.with(self.theme.primary)
        };

        println!(
            "  {} {: <9} {: <10} {}",
            icon,
            hash_styled,
            time.with(self.theme.text_dim),
            msg.with(self.theme.text)
        );
    }

    pub fn row_version(&self, index: usize, hash: &str, time: &str, is_latest: bool) {
        let icon = if is_latest {
            "●".with(self.theme.success)
        } else {
            "·".with(self.theme.text_dim)
        };

        println!(
            "    {} {: <4} {: <8} {}",
            icon,
            format!("#{}", index).with(self.theme.secondary).bold(),
            hash.with(self.theme.timeline_purple).bold().underlined(),
            time.with(self.theme.text_dim).italic()
        );
    }

    pub fn row_history_compact(
        &self,
        hash: &str,
        action: &str,
        file_path: &str,
        time: &str,
        is_latest: bool,
        diff_stats: Option<(usize, usize)>,
    ) {
        let (icon, color) = if is_latest {
            ("●", self.theme.success)
        } else {
            ("·", self.theme.text_dim)
        };

        // Compact Action Codes: A=Add, M=Mod, D=Del, C=Chk, G=Git
        let (act_code, act_color) = match action {
            "C" | "create" | "A" | "add" => ("A", self.theme.success), // Green
            "M" | "modify" => ("M", self.theme.timeline_yellow),       // Yellow
            "D" | "delete" => ("D", self.theme.error),                 // Red
            "CP" | "checkpoint" => ("C", self.theme.timeline_purple),  // Purple
            "GIT" | "commit" => ("G", self.theme.text_dim),            // Gray
            _ => ("?", self.theme.text_dim),
        };

        let diff_text = if let Some((added, removed)) = diff_stats {
            format!(
                "{} {}",
                format!("+{}", added).with(self.theme.success),
                format!("-{}", removed).with(self.theme.error)
            )
        } else {
            String::new()
        };

        println!(
            "  {} {: <8} {} {: <35} {: <10} {}",
            icon.with(color),
            hash.with(self.theme.timeline_purple).bold().underlined(),
            act_code.with(act_color).bold(),
            file_path.with(self.theme.text_bright),
            diff_text,
            time.with(self.theme.text_dim).italic()
        );
    }

    pub fn header_cyan(&self, title: &str) {
        println!();
        println!(
            "  {}",
            title.to_uppercase().with(self.theme.text_bright).bold()
        );
        let bar = "─".repeat(title.len() + 4);
        println!("  {}", bar.with(self.theme.border));
        // Removed vertical line
    }

    pub fn section_branch(&self, code: &str, title: &str) {
        println!();
        println!(
            "  {}  {}",
            code.to_uppercase().with(self.theme.secondary).bold(),
            title.with(self.theme.text_bright).bold()
        );
        println!("  {}", "┄".repeat(40).with(self.theme.border_dim));
    }

    pub fn row_search_match(&self, line_num: usize, content: &str) {
        let content_trimmed = content.trim();
        println!(
            "    {: <6} {}",
            format!("L{}", line_num).with(self.theme.text_dim),
            content_trimmed.with(self.theme.text)
        );
    }

    pub fn row_file_path(&self, path: &str) {
        println!("    {}", path.with(self.theme.text_dim));
    }

    pub fn row_status(&self, label: &str, value: &str) {
        let val_color = if value.to_lowercase().contains("active")
            || value.to_lowercase().contains("on")
        {
            self.theme.success
        } else if value.to_lowercase().contains("inactive") || value.to_lowercase().contains("off")
        {
            self.theme.error
        } else {
            self.theme.info_bright
        };

        println!(
            "    {: <15} {}",
            label.with(self.theme.text_dim),
            value.with(val_color).bold()
        );
    }

    pub fn row_key_value(&self, key: &str, value: &str) {
        println!(
            "    {: <15} {}",
            key.with(self.theme.text_dim),
            value.with(self.theme.text_bright)
        );
    }

    pub fn row_labeled(&self, icon: &str, label: &str, value: &str) {
        let display_icon = if icon.is_empty() {
            "◆".to_string()
        } else {
            icon.to_string()
        };
        println!(
            "    {} {: <14} {}",
            display_icon.with(self.theme.warning),
            label.with(self.theme.secondary),
            value.with(self.theme.text_bright).bold()
        );
    }

    pub fn row_metric(&self, icon: &str, label: &str, value: &str) {
        // Removed icon for cleaner look, focusing on label alignment
        println!(
            "    {: <15} {}",
            label.with(self.theme.text_dim),
            value.with(self.theme.text_bright).bold()
        );
    }

    pub fn row_metric_purple(&self, icon: &str, label: &str, value: &str) {
        println!(
            "{} {} {} {}",
            "│".with(self.theme.timeline_purple),
            icon.with(self.theme.timeline_purple),
            label.with(self.theme.text_muted),
            value.with(self.theme.timeline_cyan).bold()
        );
    }

    pub fn row_metric_orange(&self, icon: &str, label: &str, value: &str) {
        println!(
            "{} {} {} {}",
            "│".with(self.theme.timeline_orange),
            icon.with(self.theme.timeline_orange),
            label.with(self.theme.text_muted),
            value.with(self.theme.timeline_pink).bold()
        );
    }

    pub fn row_tag(&self, tag: &str, content: &str) {
        println!(
            "{}  [{}] {}",
            "│".with(self.theme.text),
            tag.with(self.theme.timeline_purple).bold(),
            content.with(self.theme.text)
        );
    }

    pub fn footer_hint(&self, hint: &str) {
        println!();
        println!(
            "  {} {}",
            "💡".with(self.theme.warning),
            hint.with(self.theme.text_muted)
        );
    }

    pub fn header(&self, title: &str) {
        self.header_dashboard(title);
    }

    pub fn section_start(&self, label: &str, title: &str) {
        self.section_timeline(label, title);
    }

    pub fn row_property(&self, key: &str, value: &str) {
        self.row_key_value(key, value);
    }

    pub fn row_list(&self, _icon: &str, content: &str) {
        println!("    {}", content.with(self.theme.text));
    }

    pub fn success(&self, message: &str) {
        println!(
            "    {} {}",
            "✓".with(self.theme.success),
            message.with(self.theme.success)
        );
    }

    pub fn success_bright(&self, message: &str) {
        println!(
            "  {}  {} {}",
            "╰─".with(self.theme.success_bright),
            "●".with(self.theme.success_bright),
            message.with(self.theme.text_bright).bold()
        );
    }

    pub fn warning(&self, message: &str) {
        println!(
            "    {} {}",
            "⚠".with(self.theme.warning),
            message.with(self.theme.warning)
        );
    }

    pub fn error(&self, message: &str) {
        println!(
            "    {} {}",
            "✗".with(self.theme.error),
            message.with(self.theme.error)
        );
    }

    pub fn error_bright(&self, message: &str) {
        println!(
            "  {}  {} {}",
            "╰─".with(self.theme.error_bright),
            "●".with(self.theme.error_bright),
            message.with(self.theme.text_bright).bold()
        );
    }

    pub fn info(&self, message: &str) {
        println!(
            "    {} {}",
            "ℹ".with(self.theme.info),
            message.with(self.theme.info)
        );
    }

    pub fn info_bright(&self, message: &str) {
        println!(
            "  {}  {} {}",
            "╰─".with(self.theme.info_bright),
            "●".with(self.theme.info_bright),
            message.with(self.theme.info_bright).bold()
        );
    }

    pub fn item_simple(&self, content: &str) {
        println!("    {}", content.with(self.theme.text));
    }

    pub fn item_accent(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.accent),
            content.with(self.theme.accent)
        );
    }

    pub fn item_cyan(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.timeline_cyan),
            content.with(self.theme.timeline_cyan)
        );
    }

    pub fn item_pink(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.timeline_pink),
            content.with(self.theme.timeline_pink)
        );
    }

    pub fn item_green(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.timeline_green),
            content.with(self.theme.timeline_green)
        );
    }

    pub fn item_orange(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.timeline_orange),
            content.with(self.theme.timeline_orange)
        );
    }

    pub fn item_purple(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.timeline_purple),
            content.with(self.theme.timeline_purple)
        );
    }

    pub fn item_yellow(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.timeline_yellow),
            content.with(self.theme.timeline_yellow)
        );
    }

    pub fn row_snapshot(&self, hash: &str, content: &str) {
        self.row_history(hash, content, "", false);
    }

    pub fn row_snapshot_latest(&self, hash: &str, content: &str) {
        self.row_history(hash, content, "", true);
    }

    pub fn legend(&self, items: &[(&str, &str)]) {
        let mut legend = String::new();
        for (icon, desc) in items {
            if !legend.is_empty() {
                legend.push_str("   ");
            }
            legend.push_str(&format!(
                "{} {}",
                icon.with(self.theme.text_bright).bold(),
                desc.with(self.theme.text_dim)
            ));
        }
        println!();
        println!("  {}", legend);
    }

    pub fn row_diff_add(&self, content: &str) {
        println!(
            "{}   + {}",
            "│".with(self.theme.diff_add),
            content.with(self.theme.diff_add)
        );
    }

    pub fn row_diff_remove(&self, content: &str) {
        println!(
            "{}   - {}",
            "│".with(self.theme.diff_remove),
            content.with(self.theme.diff_remove)
        );
    }

    pub fn row_diff_context(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.diff_context),
            content.with(self.theme.diff_context)
        );
    }

    pub fn row_diff_header(&self, content: &str) {
        println!(
            "{}   {}",
            "│".with(self.theme.diff_header),
            content.with(self.theme.diff_header).bold()
        );
    }

    pub fn footer(&self, hint: &str) {
        self.footer_hint(hint);
    }

    pub fn footer_pagination(&self, shown: usize, total: usize, limit: usize) {
        if total > limit {
            println!();
            println!(
                "  {} Showing {} of {} items. Use --limit N to see more.",
                "ℹ".with(self.theme.info),
                shown.to_string().with(self.theme.text_bright).bold(),
                total.to_string().with(self.theme.text_bright).bold()
            );
        }
    }

    pub fn usage(&self, command: &str, usage: &str) {
        println!(
            "    {} {}",
            "Usage:".with(self.theme.accent).bold(),
            format!("mnem {} {}", command, usage).with(self.theme.text)
        );
    }

    pub fn separator(&self) {}

    pub fn empty(&self) {
        println!();
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn theme_mut(&mut self) -> &mut Theme {
        &mut self.theme
    }

    pub fn badge(&self, label: &str, value: &str) {
        println!(
            "{}  {} {}",
            "╰─".with(self.theme.text_muted),
            format!("[{}]", label).with(self.theme.accent).bold(),
            value.with(self.theme.text_bright)
        );
    }

    pub fn badge_success(&self, label: &str, value: &str) {
        println!(
            "{}  {} {}",
            "╰─".with(self.theme.success),
            format!("[{}]", label).with(self.theme.success).bold(),
            value.with(self.theme.text_bright)
        );
    }

    pub fn badge_error(&self, label: &str, value: &str) {
        println!(
            "{}  {} {}",
            "╰─".with(self.theme.error),
            format!("[{}]", label).with(self.theme.error).bold(),
            value.with(self.theme.text_bright)
        );
    }

    pub fn badge_info(&self, label: &str, value: &str) {
        println!(
            "{}  {} {}",
            "╰─".with(self.theme.info),
            format!("[{}]", label).with(self.theme.info).bold(),
            value.with(self.theme.text_bright)
        );
    }

    pub fn bullet(&self, content: &str) {
        println!(
            "{}  ● {}",
            "│".with(self.theme.text_muted),
            content.with(self.theme.text)
        );
    }

    pub fn bullet_cyan(&self, content: &str) {
        println!(
            "{}  • {}",
            "│".with(self.theme.timeline_cyan),
            content.with(self.theme.timeline_cyan)
        );
    }

    pub fn bullet_pink(&self, content: &str) {
        println!(
            "{}  • {}",
            "│".with(self.theme.timeline_pink),
            content.with(self.theme.timeline_pink)
        );
    }

    pub fn bullet_green(&self, content: &str) {
        println!(
            "{}  • {}",
            "│".with(self.theme.timeline_green),
            content.with(self.theme.timeline_green)
        );
    }

    pub fn bullet_orange(&self, content: &str) {
        println!(
            "{}  • {}",
            "│".with(self.theme.timeline_orange),
            content.with(self.theme.timeline_orange)
        );
    }

    pub fn bullet_purple(&self, content: &str) {
        println!(
            "{}  • {}",
            "│".with(self.theme.timeline_purple),
            content.with(self.theme.timeline_purple)
        );
    }

    pub fn bullet_yellow(&self, content: &str) {
        println!(
            "{}  • {}",
            "│".with(self.theme.timeline_yellow),
            content.with(self.theme.timeline_yellow)
        );
    }

    pub fn title(&self, text: &str) {
        println!("  {}", text.with(self.theme.primary).bold());
    }

    pub fn subtitle(&self, text: &str) {
        println!("  {}", text.with(self.theme.accent));
    }

    pub fn key(&self, text: &str) {
        print!("{} ", text.with(self.theme.timeline_pink).bold());
    }

    pub fn value(&self, text: &str) {
        println!("{}", text.with(self.theme.text));
    }

    pub fn value_accent(&self, text: &str) {
        println!("{}", text.with(self.theme.accent));
    }

    pub fn value_success(&self, text: &str) {
        println!("{}", text.with(self.theme.success));
    }

    pub fn value_error(&self, text: &str) {
        println!("{}", text.with(self.theme.error));
    }

    pub fn value_info(&self, text: &str) {
        println!("{}", text.with(self.theme.info));
    }

    pub fn row_version_with_link(
        &self,
        index: usize,
        hash: &str,
        full_hash: &str,
        file_path: &str,
        time: &str,
        is_latest: bool,
        diff_stats: Option<(usize, usize)>,
        ide: &mnem_core::config::Ide,
    ) {
        let icon = if is_latest {
            "●".with(self.theme.success)
        } else {
            "·".with(self.theme.text_dim)
        };

        let link = Hyperlink::ide_link(hash, file_path, ide);

        let diff_text = if let Some((added, removed)) = diff_stats {
            format!(
                "{} {}",
                format!("+{}", added).with(self.theme.success),
                format!("-{}", removed).with(self.theme.error)
            )
        } else {
            String::new()
        };

        println!(
            "    {} {: <4} {} {: <10} {}",
            icon,
            format!("#{}", index).with(self.theme.secondary).bold(),
            link.with(if is_latest {
                self.theme.success
            } else {
                self.theme.timeline_purple
            })
            .bold()
            .underlined(),
            diff_text,
            time.with(self.theme.text_dim).italic(),
        );
    }

    pub fn row_hash_link(
        &self,
        hash: &str,
        full_hash: &str,
        file_path: &str,
        ide: &mnem_core::config::Ide,
    ) {
        let link = Hyperlink::ide_link(hash, file_path, ide);
        println!(
            "{}   {}",
            "│".with(self.theme.timeline_cyan),
            link.with(self.theme.accent)
        );
    }
}
