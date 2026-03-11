use crate::commands::common::{CommandStrategy, GlobalOptions};
use crate::ui::presentable::Renderable;
use crate::ui::{
    Banner, BranchBadge, Breadcrumbs, Card, CodeBlock, Column, DiffBar, DiffView, Gauge, Highlight,
    Hyperlink, KeyHint, List, Messages, MetadataGrid, PaginationInfo, Spinner, StepProgress, Table,
    Timeline,
};
use anyhow::Result;
use clap::Args;
use mnem_core::protocol::SnapshotInfo;
use std::path::PathBuf;

/// Debug command to display all UI components
#[derive(Args)]
pub struct ComponentsCommand {}

impl CommandStrategy for ComponentsCommand {
    fn execute(&self, _global_opts: &GlobalOptions) -> Result<()> {
        let theme = crate::theme::Theme::from_mnemosyne();
        let messages = Messages::new(theme.clone());

        // 1. Banner
        let banner = Banner::new(theme.clone());
        banner.show("MNEMOSYNE UI COMPONENTS");

        // 2. Timeline (Mocked)
        messages.info("Timeline (Chronological Activity):");
        let mock_snapshots = vec![
            SnapshotInfo {
                id: 1,
                content_hash: "a7f2d31b2c5d11".into(),
                timestamp: "2026-03-10T14:30:00Z".into(),
                file_path: "src/lib.rs".into(),
                git_branch: Some("main".into()),
                commit_hash: Some("abc1234".into()),
                commit_message: Some("feat: add rpc handler".into()),
                checkpoint_name: None,
            },
            SnapshotInfo {
                id: 2,
                content_hash: "f3e1a02b2c5d11".into(),
                timestamp: "2026-03-10T14:32:00Z".into(),
                file_path: "src/main.rs".into(),
                git_branch: Some("main".into()),
                commit_hash: None,
                commit_message: None,
                checkpoint_name: Some("stable-build".into()),
            },
        ];
        let timeline = Timeline::new(
            "Mock Timeline",
            mock_snapshots,
            PathBuf::from("/"),
            Some("src/lib.rs".into()),
        );
        timeline.text()?;
        println!();

        // 3. DiffView
        messages.info("DiffView:");
        let diff = DiffView::new(theme.clone());
        diff.header("crates/core/lib.rs");
        diff.render_line(" pub fn snapshot() {", " ");
        diff.render_line("-    let old = 1;", "-");
        diff.render_line("+    let new = 2;", "+");
        diff.render_line(" }", " ");
        println!();

        // 4. Breadcrumbs
        messages.info("Breadcrumbs:");
        let breadcrumbs = Breadcrumbs::new(theme.clone());
        breadcrumbs.show(&[
            ("󱂵", "mnemosyne"),
            ("󱞩", "projects"),
            ("", "core-daemon"),
            ("", "main"),
        ]);
        println!();

        // 5. Card
        messages.info("Card:");
        let card = Card::new(theme.clone(), "Project Summary");
        card.render(vec![
            ("Path", "/users/mnem/project".to_string()),
            ("Items", "1,402 snapshots".to_string()),
            ("Size", "432.1 MB".to_string()),
        ]);
        println!();

        // 6. Table (Smart & Symbolic with Pagination)
        messages.info("Table (Smart & Symbolic with Pagination):");
        let table = Table::new(
            theme.clone(),
            vec![
                Column::new("TYPE", 4),
                Column::new("HASH", 8),
                Column::flex("FILE PATH", 25),
                Column::new("SIZE", 8),
            ],
        );
        table.header();
        table.row(&[
            "◈".to_string(),
            "a7f2d31".to_string(),
            "crates/core/lib.rs".to_string(),
            "12.4 KB".to_string(),
        ]);
        table.row(&[
            "●".to_string(),
            "f3e1a02".to_string(),
            "crates/apps/main.rs".to_string(),
            "4.2 KB".to_string(),
        ]);
        let table_pagination =
            PaginationInfo::new(1, 42, 20).with_info("FILTER".to_string(), "*.rs".to_string());
        table.pagination(&table_pagination);
        println!();

        // 7. DiffBar
        messages.info("DiffBar:");
        let diff_bar = DiffBar::new(theme.clone());
        diff_bar.render(42, 12);
        diff_bar.render(5, 20);
        println!();

        // 8. Gauge
        messages.info("Gauge:");
        let gauge = Gauge::new(theme.clone());
        gauge.render("CAS Health", 0.78);
        gauge.render("Disk Usage", 0.92);
        println!();

        // 9. StepProgress
        messages.info("StepProgress:");
        let step = StepProgress::new(theme.clone());
        step.render(&[
            ("Connect", true, false),
            ("Scan", false, true),
            ("Repair", false, false),
            ("Done", false, false),
        ]);
        println!();

        // 10. CodeBlock
        messages.info("CodeBlock:");
        let code = CodeBlock::new(theme.clone());
        code.render(
            &[
                "pub fn main() {",
                "    let mnem = Mnemosyne::new();",
                "    mnem.snapshot();",
                "}",
            ],
            10,
        );
        println!();

        // 11. MetadataGrid
        messages.info("MetadataGrid:");
        let grid = MetadataGrid::new(theme.clone());
        grid.render(vec![
            ("HASH", "a7f2d31".to_string()),
            ("BRANCH", "rpc-refactor".to_string()),
            ("AUTHOR", "@tacosalfornoh".to_string()),
        ]);
        println!();

        // 12. List & PaginationInfo
        messages.info("List & PaginationInfo:");
        let list = List::new(theme.clone());
        list.bullet("First list item");
        list.numbered(2, "Second numbered item");
        let list_pagination =
            PaginationInfo::new(1, 100, 20).with_info("STATUS".to_string(), "Testing".to_string());
        list.pagination(&list_pagination);
        println!();

        // 13. KeyHint
        messages.info("KeyHint:");
        let key_hint = KeyHint::new(theme.clone());
        key_hint.show(&[
            ("ENTER", "view"),
            ("s", "snapshot"),
            ("d", "diff"),
            ("q", "quit"),
        ]);
        println!();

        // 14. Highlight
        messages.info("Highlight:");
        let highlighter = Highlight::new(theme.clone());
        let highlighted = highlighter.text("Searching for Mnemosyne in this text", "Mnemosyne");
        println!("  {}", highlighted);
        println!();

        // 15. Elements (Badge & Hyperlink)
        messages.info("Elements (Badge & Hyperlink):");
        println!("  Badge: {}", BranchBadge::simple("feature/ui-refactor"));
        println!(
            "  Link:  {}",
            Hyperlink::new("Open Spec", "https://mnemosyne.dev")
        );
        println!();

        // 16. Messages
        messages.info("Messages:");
        messages.success("Operation successful");
        messages.warning("Warning message");
        messages.error("Error occurred");
        messages.debug("Debug information");
        println!();

        // 17. Spinner (Static example)
        messages.info("Spinner:");
        let spinner = Spinner::new(theme.clone());
        spinner.frame(0, "Connecting to daemon...");
        println!();
        println!();

        Ok(())
    }
}
