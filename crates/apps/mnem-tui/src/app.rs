use crate::theme::{Theme, THEMES};
use mnem_core::config::Config;
use mnem_core::models::{FileEntry, SearchResult, Snapshot};
use ratatui::text::Line;
use ratatui::widgets::ListState;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ViewState {
    Home,
    History,
    Settings,
    Search,
    Projects,
    Statistics,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Focus {
    Files,
    Timeline,
    Preview,
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub label: String,
    pub branch: Option<String>,
    pub count: usize,
}

#[derive(Clone, Debug)]
pub enum HistoryItem {
    DateHeader(String),
    Session(SessionInfo),
    Snapshot(Snapshot),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DialogType {
    BranchSelector,
    Confirmation {
        title: &'static str,
        message: &'static str,
    },
}

pub struct AppState {
    pub view: ViewState,
    pub focus: Focus,
    pub show_dialog: Option<DialogType>,
    pub dialog_state: ListState,

    // Project Info
    pub project_name: String,
    pub git_branch: Option<String>,

    // Branch Filter
    pub branch_filter: Option<String>,
    pub available_branches: Vec<String>,

    // Time Filter (reserved for future use)
    pub time_filter: Option<String>,

    // Dirty flag: when true, data needs to be re-fetched from DB
    pub dirty: bool,
    // Preview dirty: when true, diff needs to be recalculated
    pub preview_dirty: bool,
    // Last selection time for debouncing
    pub last_selection_time: std::time::Instant,

    // File List
    pub files_state: ListState,
    pub files: Vec<FileEntry>,

    // Timeline (Work Sessions)
    pub versions_state: ListState,
    pub history_items: Vec<HistoryItem>,

    // Search
    pub search_state: ListState,
    pub search_results: Vec<SearchResult>,
    pub search_query: String,

    // Selection
    pub selected_file: Option<String>,
    pub diff_base_hash: Option<String>,

    // Diff Cache
    pub cached_diff: Vec<Line<'static>>,
    pub last_diff_hash: Option<String>,
    pub selected_hunks: std::collections::HashSet<usize>,
    pub hunk_count: usize,
    pub active_hunk_index: Option<usize>,

    // UI
    pub filter_input: String,
    pub input_mode: bool,
    pub scroll_offset: u32,
    pub theme: Theme,

    // Feedback
    pub notification: Option<(String, std::time::Instant)>,

    // Settings
    pub settings_index: usize,
    pub config: Config,

    // Preview Selection
    pub preview_state: ListState,

    pub current_preview_hash: Option<String>,
    pub current_lang: Option<String>,
    pub diff_plus: usize,
    pub diff_minus: usize,
    pub last_snapshot_id: i64,

    // Projects View
    pub projects: Vec<mnem_core::protocol::WatchedProject>,
    pub projects_state: ListState,

    // Statistics View
    pub stats: Option<mnem_core::protocol::ProjectStatisticsResponse>,

    // Layout
    pub sidebar_width_pct: u16,
    pub timeline_width_pct: u16,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: ViewState::Home,
            focus: Focus::Files,
            show_dialog: None,
            dialog_state: ListState::default(),
            project_name: "Loading...".to_string(),
            git_branch: None,
            branch_filter: None,
            available_branches: Vec::new(),
            time_filter: None,
            dirty: true,
            preview_dirty: true,
            last_selection_time: std::time::Instant::now(),
            files_state: ListState::default(),
            files: Vec::new(),
            versions_state: ListState::default(),
            history_items: Vec::new(),
            search_state: ListState::default(),
            search_results: Vec::new(),
            search_query: String::new(),
            selected_file: None,
            diff_base_hash: None,
            cached_diff: Vec::new(),
            last_diff_hash: None,
            selected_hunks: std::collections::HashSet::new(),
            hunk_count: 0,
            active_hunk_index: None,
            filter_input: String::new(),
            input_mode: false,
            scroll_offset: 0,
            theme: THEMES[0].clone(),
            notification: None,
            settings_index: 0,
            config: Config::default(),
            preview_state: ListState::default(),
            current_preview_hash: None,
            current_lang: None,
            diff_plus: 0,
            diff_minus: 0,
            last_snapshot_id: 0,
            projects: Vec::new(),
            projects_state: ListState::default(),
            stats: None,
            sidebar_width_pct: 35,
            timeline_width_pct: 50,
        }
    }
}

impl AppState {
    pub fn set_notification(&mut self, msg: String) {
        self.notification = Some((msg, std::time::Instant::now()));
    }

    pub fn clear_expired_notifications(&mut self) {
        if let Some((_, time)) = self.notification {
            if time.elapsed() > std::time::Duration::from_secs(3) {
                self.notification = None;
            }
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}
