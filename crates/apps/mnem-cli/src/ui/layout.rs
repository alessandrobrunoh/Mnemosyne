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
        println!("  {}", title.to_uppercase().with(self.theme.info).bold());
        let bar = "━".repeat(title.len() + 4);
        println!("  {}", bar.with(self.theme.info));
        println!("{}", "│".with(self.theme.text_bright));
    }

    pub fn section_timeline(&self, code: &str, title: &str) {
        println!(
            "{}╭┄ {} [{}]",
            "│".with(self.theme.text_bright),
            code.with(self.theme.accent).bold(),
            title.with(self.theme.text_bright).bold()
        );
    }

    pub fn section_end(&self) {
        println!("{}", "├╯".with(self.theme.text_bright));
        println!("{}", "│".with(self.theme.text_bright));
    }

    pub fn row_timeline(&self, icon: &str, content: &str) {
        println!("{}  {} {}", "┊".with(self.theme.text), icon, content);
    }

    pub fn row_history(&self, hash: &str, time: &str, msg: &str, is_latest: bool) {
        let icon = if is_latest {
            "●".with(self.theme.success)
        } else {
            "●".with(self.theme.text_muted)
        };
        let hash_styled = if is_latest {
            hash.with(self.theme.success).bold()
        } else {
            hash.with(self.theme.accent)
        };

        println!(
            "{}   {}   {: <8} {: <16} {}",
            "┊".with(self.theme.text),
            icon,
            hash_styled,
            time.with(self.theme.text_muted),
            msg.with(self.theme.text)
        );
    }

    pub fn row_version(&self, index: usize, hash: &str, time: &str, is_latest: bool) {
        let icon = if is_latest {
            "●".with(self.theme.success)
        } else {
            "●".with(self.theme.text_muted)
        };

        println!(
            "{}   {}   {: <4} {: <8} {}",
            "│".with(self.theme.text),
            icon,
            format!("#{}", index).with(self.theme.primary).bold(),
            hash.with(self.theme.text_muted),
            time.with(self.theme.text)
        );
    }

    pub fn row_history_compact(
        &self,
        hash: &str,
        action: &str,
        file_path: &str,
        time: &str,
        is_latest: bool,
    ) {
        let icon = if is_latest {
            "●".with(self.theme.primary)
        } else {
            "●".with(self.theme.text_muted)
        };

        let action_colored = match action {
            "C" | "create" => "C".with(self.theme.timeline_cyan).bold(),
            "M" | "modify" => "M".with(self.theme.timeline_orange).bold(),
            "D" | "delete" => "D".with(self.theme.error).bold(),
            _ => action.with(self.theme.text_muted),
        };

        println!(
            "{} {} {} {} {} {}",
            "│".with(self.theme.text),
            icon,
            hash.with(self.theme.primary).bold(),
            action_colored,
            file_path.with(self.theme.text),
            time.with(self.theme.text_muted)
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
        println!("{}", "│".with(self.theme.text));
    }

    pub fn section_branch(&self, code: &str, title: &str) {
        use crossterm::style::Color;
        let violet = Color::Rgb {
            r: 189,
            g: 147,
            b: 249,
        };
        println!(
            "{}╭┄ {} [{}]",
            "│".with(self.theme.text_bright),
            code.with(violet).bold(),
            title.with(self.theme.text_bright).bold()
        );
    }

    pub fn row_search_match(&self, line_num: usize, content: &str) {
        let content_trimmed = content.trim();
        println!(
            "{}   {} {}",
            "┊".with(self.theme.text),
            format!("L{}", line_num).with(self.theme.text_muted),
            content_trimmed.with(self.theme.text)
        );
    }

    pub fn row_file_path(&self, path: &str) {
        println!(
            "{}   {}",
            "┊".with(self.theme.text),
            path.with(self.theme.text_muted)
        );
    }

    pub fn row_status(&self, label: &str, value: &str) {
        println!(
            "{} {} {}",
            "│".with(self.theme.text_bright),
            label.with(self.theme.text_muted),
            value.with(self.theme.text_bright).bold()
        );
    }

    pub fn row_key_value(&self, key: &str, value: &str) {
        println!(
            "{}  {} {}",
            "│".with(self.theme.text_bright),
            key.with(self.theme.text_muted).bold(),
            value.with(self.theme.text_bright)
        );
    }

    pub fn row_labeled(&self, icon: &str, label: &str, value: &str) {
        if icon.is_empty() {
            println!(
                "{} {} {}",
                "│".with(self.theme.text_bright),
                label.with(self.theme.text_muted),
                value.with(self.theme.text_bright).bold()
            );
        } else {
            println!(
                "{} {} {} {}",
                "│".with(self.theme.text_bright),
                icon.with(self.theme.text),
                label.with(self.theme.text_muted),
                value.with(self.theme.text_bright).bold()
            );
        }
    }

    pub fn row_metric(&self, icon: &str, label: &str, value: &str) {
        if icon.is_empty() {
            println!(
                "{} {} {}",
                "│".with(self.theme.text_bright),
                label.with(self.theme.text_muted),
                value.with(self.theme.text_bright).bold()
            );
        } else {
            println!(
                "{} {} {} {}",
                "│".with(self.theme.text_bright),
                icon.with(self.theme.text),
                label.with(self.theme.text_muted),
                value.with(self.theme.text_bright).bold()
            );
        }
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
        println!(
            "{}   {}",
            "┊".with(self.theme.text),
            content.with(self.theme.text)
        );
    }

    pub fn success(&self, message: &str) {
        println!(
            "{}   {} {}",
            "┊".with(self.theme.text),
            "✓".with(self.theme.success),
            message.with(self.theme.success)
        );
    }

    pub fn success_bright(&self, message: &str) {
        println!(
            "{}   {} {}",
            "├╯".with(self.theme.success_bright),
            "●".with(self.theme.success_bright),
            message.with(self.theme.success_bright).bold()
        );
    }

    pub fn warning(&self, message: &str) {
        println!(
            "{}   {} {}",
            "┊".with(self.theme.text),
            "⚠".with(self.theme.warning),
            message.with(self.theme.warning)
        );
    }

    pub fn error(&self, message: &str) {
        println!(
            "{}   {} {}",
            "┊".with(self.theme.text),
            "✗".with(self.theme.error),
            message.with(self.theme.error)
        );
    }

    pub fn error_bright(&self, message: &str) {
        println!(
            "{}   {} {}",
            "├╯".with(self.theme.error_bright),
            "⬤".with(self.theme.error_bright),
            message.with(self.theme.error_bright).bold()
        );
    }

    pub fn info(&self, message: &str) {
        println!(
            "{}   {} {}",
            "┊".with(self.theme.text),
            "ℹ".with(self.theme.info),
            message.with(self.theme.info)
        );
    }

    pub fn info_bright(&self, message: &str) {
        println!(
            "{}   {} {}",
            "├╯".with(self.theme.info_bright),
            "◆".with(self.theme.info_bright),
            message.with(self.theme.info_bright).bold()
        );
    }

    pub fn item_simple(&self, content: &str) {
        println!(
            "{}   {}",
            "┊".with(self.theme.text),
            content.with(self.theme.text)
        );
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
                icon.with(self.theme.text_muted),
                desc.with(self.theme.text_muted)
            ));
        }
        println!("{}   {}", "┊".with(self.theme.text), legend);
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

    pub fn usage(&self, command: &str, usage: &str) {
        println!(
            "{}   {} {}",
            "┊".with(self.theme.text),
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
            "┄".with(self.theme.text),
            format!("[{}]", label).with(self.theme.accent).bold(),
            value.with(self.theme.text)
        );
    }

    pub fn badge_success(&self, label: &str, value: &str) {
        println!(
            "{}  {} {}",
            "┄".with(self.theme.success),
            format!("[{}]", label).with(self.theme.success).bold(),
            value.with(self.theme.text)
        );
    }

    pub fn badge_error(&self, label: &str, value: &str) {
        println!(
            "{}  {} {}",
            "┄".with(self.theme.error),
            format!("[{}]", label).with(self.theme.error).bold(),
            value.with(self.theme.text)
        );
    }

    pub fn badge_info(&self, label: &str, value: &str) {
        println!(
            "{}  {} {}",
            "┄".with(self.theme.info),
            format!("[{}]", label).with(self.theme.info).bold(),
            value.with(self.theme.text)
        );
    }

    pub fn bullet(&self, content: &str) {
        println!(
            "{}  • {}",
            "│".with(self.theme.accent),
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
        ide: &mnem_core::config::Ide,
    ) {
        let icon = if is_latest {
            "●".with(self.theme.success)
        } else {
            "●".with(self.theme.text_muted)
        };

        let link = Hyperlink::ide_link(hash, file_path, ide);

        println!(
            "{}   {}   {: <4} {} {: <16}",
            "┊".with(self.theme.text),
            icon,
            format!("#{}", index).with(self.theme.accent).bold(),
            link.with(if is_latest {
                self.theme.success
            } else {
                self.theme.accent
            }),
            time.with(self.theme.text_muted),
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
