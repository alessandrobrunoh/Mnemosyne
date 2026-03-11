use anyhow::Result;
use crossterm::style::Stylize;
use std::path::{Path, PathBuf};

use crate::ui::{DiffBar, KeyHint, Layout, List, PaginationInfo, Renderable, Table};
use mnem_core::protocol::SnapshotInfo;

/// Timeline component for displaying a chronological sequence of snapshots
pub struct Timeline {
    pub title: String,
    pub snapshots: Vec<SnapshotInfo>,
    pub project_path: PathBuf,
    pub target_file: Option<String>,
    pub limit: usize,
    pub page: usize,
}

impl Timeline {
    pub fn new(
        title: &str,
        snapshots: Vec<SnapshotInfo>,
        project_path: PathBuf,
        target_file: Option<String>,
    ) -> Self {
        Self {
            title: title.to_string(),
            snapshots,
            project_path,
            target_file,
            limit: 20,
            page: 1,
        }
    }

    fn format_time(timestamp: &str) -> String {
        if timestamp.len() > 16 {
            timestamp[11..16].to_string()
        } else {
            timestamp.to_string()
        }
    }

    /// Display pagination information footer using the Table component's logic
    /// or a direct consistent implementation.
    pub fn pagination(&self, info: &PaginationInfo, theme: &crate::theme::Theme) {
        let table = Table::new(theme.clone(), vec![]);
        table.pagination(info);
    }
}

impl Renderable for Timeline {
    fn text(&self) -> Result<()> {
        let layout = Layout::new();
        let theme = layout.theme();

        if self.snapshots.is_empty() {
            layout.info("No activity found.");
            return Ok(());
        }

        let total_count = self.snapshots.len();
        let offset = (self.page - 1) * self.limit;

        let paged_items: Vec<&SnapshotInfo> = self
            .snapshots
            .iter()
            .skip(offset)
            .take(self.limit)
            .collect();

        if paged_items.is_empty() && self.page > 1 {
            layout.warning(&format!(
                "Page {} is empty. Total items: {}",
                self.page, total_count
            ));
            return Ok(());
        }

        let mut first_branch = true;
        let mut current_block_id: Option<String> = None;
        let _diff_bar = DiffBar::new(theme.clone());

        for (idx, snap) in paged_items.iter().enumerate() {
            let _global_idx = offset + idx + 1;
            let hash = snap.content_hash.get(0..7).unwrap_or("unknown");
            let time = Self::format_time(&snap.timestamp);
            let is_latest = self.page == 1 && idx == 0;

            // 1. Detect Branch Change
            let branch_name = snap.git_branch.as_deref().unwrap_or("main");
            if first_branch {
                layout.graph_branch_start(branch_name);
                first_branch = false;
            }

            // 2. Detect Semantic Block Change (Git Commit or Checkpoint)
            let block_id = if let Some(ref ch) = snap.commit_hash {
                Some(format!("git:{}", ch))
            } else if let Some(ref cp) = snap.checkpoint_name {
                Some(format!("cp:{}", cp))
            } else {
                None
            };

            if block_id != current_block_id {
                if current_block_id.is_some() {
                    layout.graph_connector();
                }

                if let Some(ref _ch) = snap.commit_hash {
                    let msg = snap.commit_message.as_deref().unwrap_or("Git Commit");
                    layout.graph_block_header("G", msg, theme.timeline_purple);
                } else if let Some(ref cp) = snap.checkpoint_name {
                    layout.graph_block_header(
                        "◈",
                        &format!("Checkpoint: {}", cp),
                        theme.timeline_cyan,
                    );
                } else if idx > 0 {
                    layout.graph_block_header("·", "Manual saves", theme.text_dim);
                }
                current_block_id = block_id;
            }

            // 3. Determine Node Info
            let (icon, color, meta) = if snap.commit_hash.is_some() {
                (
                    "G",
                    theme.timeline_purple,
                    snap.commit_message
                        .clone()
                        .unwrap_or_else(|| "Commit".into()),
                )
            } else if snap.checkpoint_name.is_some() {
                (
                    "◈",
                    theme.timeline_cyan,
                    snap.checkpoint_name.clone().unwrap_or_default(),
                )
            } else {
                (
                    "·",
                    theme.text_dim,
                    if self.target_file.is_some() {
                        "Manual save".into()
                    } else {
                        let p = Path::new(&snap.file_path);
                        p.strip_prefix(&self.project_path)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| snap.file_path.clone())
                    },
                )
            };

            let display_meta = if self.target_file.is_some() {
                format!("[{}] {}", offset + idx + 1, meta)
            } else {
                meta
            };

            layout.graph_node(hash, &display_meta, is_latest, &time, Some(icon), color);
        }

        // 4. Draw Root (Base)
        let is_last_page = offset + self.limit >= total_count;
        if is_last_page {
            if let Some(last_snap) = self.snapshots.last() {
                layout.graph_branch_end();
                layout.graph_empty_line();
                let hash = last_snap.content_hash.get(0..7).unwrap_or("init");
                let time = Self::format_time(&last_snap.timestamp);
                layout.graph_root(hash, "[base] Initial state", &time);
            }
        } else {
            layout.graph_empty_line();
            layout.graph_block_header(
                "ℹ",
                &format!("... page {} available", self.page + 1),
                theme.timeline_yellow,
            );
        }

        // 5. Discrete Footer
        let file_name = self.target_file.as_deref().unwrap_or("Project").to_string();
        let info = PaginationInfo::new(self.page, total_count, self.limit)
            .with_info("FILE".to_string(), file_name);
        self.pagination(&info, &theme);

        // 6. Shortcuts Hint
        let key_hint = KeyHint::new(theme.clone());
        key_hint.show(&[
            ("ENTER", "view"),
            ("r", "restore"),
            ("d", "diff"),
            ("q", "quit"),
        ]);

        layout.legend(&[
            ("●", "Latest"),
            ("●", "Past"),
            ("G", "Git"),
            ("◈", "Checkpoint"),
        ]);

        Ok(())
    }
}
