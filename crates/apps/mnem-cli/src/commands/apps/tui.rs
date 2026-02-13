use crate::commands::Command;
use crate::ui::TsHighlighter;
use anyhow::Result;
use chrono::{DateTime, Timelike};
use crossterm::{
    event::{self, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mnem_core::config::ConfigManager;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::Terminal;
use similar::{ChangeTag, TextDiff};
use std::sync::{Arc, MutexGuard};
use std::time::Duration;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use mnem_core::{
    client::DaemonClient,
    models::{FileEntry, Snapshot},
    protocol::methods,
    Repository,
};
use mnem_tui::app::{AppState, DialogType, Focus, HistoryItem, SessionInfo, ViewState};
use mnem_tui::theme::THEMES;
use mnem_tui::{view, AppEvent, EventHandler};

#[derive(Debug)]
pub struct TuiCommand;

impl Command for TuiCommand {
    fn name(&self) -> &str {
        "tui"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Start the interactive Text User Interface"
    }

    fn group(&self) -> &str {
        "Apps"
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let repo = Arc::new(Repository::init()?);
        start_tui(repo)
    }
}

/// Safely locks the repository config mutex, recovering from poison if necessary.
fn lock_config(repo: &Repository) -> Option<MutexGuard<'_, ConfigManager>> {
    repo.config
        .lock()
        .map_err(|poisoned| poisoned.into_inner())
        .ok()
}

pub fn start_tui(repo: Arc<Repository>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let events = EventHandler::new(Duration::from_millis(100));
    let mut state = AppState::default();
    let mut ts_highlighter = TsHighlighter::new();

    // Safely access config with poison recovery
    if let Some(config_manager) = lock_config(&repo) {
        state.config = config_manager.config.clone();
        state.theme = THEMES[config_manager.config.theme_index.min(THEMES.len() - 1)].clone();
    }

    state.files_state.select(Some(0));

    // Init Project Info
    state.project_name = repo.project.name.clone();
    state.git_branch = repo.get_current_branch();

    // Load available branches
    if let Ok(branches) = repo.list_branches() {
        state.available_branches = branches;
    }

    loop {
        // --- Logic: Data Fetching (ONLY when dirty flag is set) ---
        if state.dirty {
            match state.view {
                ViewState::Home => {
                    let filter = if !state.filter_input.is_empty() {
                        Some(state.filter_input.as_str())
                    } else {
                        None
                    };
                    let branch = state.branch_filter.as_deref();
                    if let Ok(mut client) = DaemonClient::connect() {
                        let res = client.call(
                            methods::FILE_GET_LIST,
                            serde_json::json!({
                                "filter": filter,
                                "branch": branch,
                                "limit": Some(100)
                            }),
                        )?;
                        let files: Vec<serde_json::Value> = serde_json::from_value(res)?;
                        state.files = files
                            .into_iter()
                            .filter_map(|v| {
                                Some(FileEntry {
                                    path: v["path"].as_str()?.to_string(),
                                    last_update: v["last_update"].as_str()?.to_string(),
                                })
                            })
                            .collect();
                    } else {
                        state.files = repo.list_files(filter, branch)?;
                    }

                    if state.files_state.selected().is_none() && !state.files.is_empty() {
                        state.files_state.select(Some(0));
                    }

                    // Auto-load history for first file
                    if let Some(i) = state.files_state.selected() {
                        if let Some(f) = state.files.get(i) {
                            if let Ok(snapshots) = repo.get_history(&f.path) {
                                state.history_items = group_snapshots(snapshots);
                            }
                        }
                    }
                }
                ViewState::History => {
                    if let Some(path) = &state.selected_file {
                        let snapshots = repo.get_history(path)?;
                        state.history_items = group_snapshots(snapshots);
                        state.preview_dirty = true;
                    }
                }
                ViewState::Search => {}
                ViewState::Projects => {
                    if let Ok(mut client) = DaemonClient::connect() {
                        if let Ok(res) = client.call(methods::PROJECT_LIST, serde_json::json!({})) {
                            if let Ok(resp) = serde_json::from_value::<
                                mnem_core::protocol::GetWatchedProjectsResponse,
                            >(res)
                            {
                                state.projects = resp.projects;
                                if state.projects_state.selected().is_none()
                                    && !state.projects.is_empty()
                                {
                                    state.projects_state.select(Some(0));
                                }
                            }
                        }
                    }
                }
                ViewState::Statistics => {
                    if state.stats.is_none() {
                        if let Ok(mut client) = DaemonClient::connect() {
                            if let Ok(res) = client.call(
                                methods::PROJECT_GET_STATISTICS,
                                serde_json::json!({"project_path": repo.project.path}),
                            ) {
                                if let Ok(resp) = serde_json::from_value::<
                                    mnem_core::protocol::ProjectStatisticsResponse,
                                >(res)
                                {
                                    state.stats = Some(resp);
                                }
                            }
                        }
                    }
                }
                ViewState::Settings => {
                    if let Some(config_manager) = lock_config(&repo) {
                        state.config = config_manager.config.clone();
                    }
                }
            }
            state.dirty = false;
        }

        // --- Logic: Preview (Debounced) ---
        if state.preview_dirty && state.last_selection_time.elapsed() > Duration::from_millis(300) {
            let theme_key = state.theme.name.to_lowercase().replace(" ", "-");
            let syntect_theme = ts
                .themes
                .get(&theme_key)
                .unwrap_or(&ts.themes["base16-ocean.dark"]);
            update_preview(&repo, &mut state, &ps, syntect_theme, &mut ts_highlighter)?;
            state.preview_dirty = false;
        }

        // --- Logic: Render ---
        terminal.draw(|f| view::render(f, &mut state))?;

        // --- Logic: Input ---
        match events.next()? {
            AppEvent::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::ScrollDown => match state.focus {
                    Focus::Files => {
                        next(&mut state.files_state, state.files.len(), None);
                        state.preview_dirty = true;
                        state.scroll_offset = 0;
                        state.last_selection_time = std::time::Instant::now();
                    }
                    Focus::Timeline => {
                        next(
                            &mut state.versions_state,
                            state.history_items.len(),
                            Some(&state.history_items),
                        );
                        state.preview_dirty = true;
                        state.scroll_offset = 0;
                        state.last_selection_time = std::time::Instant::now();
                    }
                    Focus::Preview => state.scroll_offset = state.scroll_offset.saturating_add(1),
                },
                MouseEventKind::ScrollUp => match state.focus {
                    Focus::Files => {
                        prev(&mut state.files_state, state.files.len(), None);
                        state.preview_dirty = true;
                        state.scroll_offset = 0;
                        state.last_selection_time = std::time::Instant::now();
                    }
                    Focus::Timeline => {
                        prev(
                            &mut state.versions_state,
                            state.history_items.len(),
                            Some(&state.history_items),
                        );
                        state.preview_dirty = true;
                        state.scroll_offset = 0;
                        state.last_selection_time = std::time::Instant::now();
                    }
                    Focus::Preview => state.scroll_offset = state.scroll_offset.saturating_sub(1),
                },
                _ => {}
            },
            AppEvent::Input(key_event) => {
                // --- Input Mode (filter/search typing) ---
                if state.input_mode {
                    match key_event.code {
                        KeyCode::Enter => {
                            state.input_mode = false;
                            if state.view == ViewState::Search {
                                if let Ok(results) = repo.grep_contents(&state.search_query, None) {
                                    state.search_results = results;
                                    state.search_state.select(Some(0));
                                }
                            } else {
                                // Filter input in Home view
                                state.dirty = true;
                            }
                        }
                        KeyCode::Esc => {
                            state.input_mode = false;
                            if state.view == ViewState::Search {
                                state.view = ViewState::Home;
                                state.dirty = true;
                            }
                        }
                        KeyCode::Backspace => {
                            if state.view == ViewState::Search {
                                state.search_query.pop();
                            } else {
                                state.filter_input.pop();
                                state.dirty = true; // Update filter
                            }
                        }
                        KeyCode::Char(c) => {
                            if state.view == ViewState::Search {
                                state.search_query.push(c);
                            } else {
                                state.filter_input.push(c);
                                state.dirty = true; // Update filter
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // --- Dialog Mode (Branch selector, etc) ---
                if let Some(dialog) = state.show_dialog {
                    match key_event.code {
                        KeyCode::Esc => state.show_dialog = None,
                        KeyCode::Down | KeyCode::Char('j') => {
                            let len = match dialog {
                                DialogType::BranchSelector => state.available_branches.len(),
                                DialogType::Confirmation { .. } => 0,
                            };
                            next(&mut state.dialog_state, len, None);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let len = match dialog {
                                DialogType::BranchSelector => state.available_branches.len(),
                                DialogType::Confirmation { .. } => 0,
                            };
                            prev(&mut state.dialog_state, len, None);
                        }
                        KeyCode::Enter => match dialog {
                            DialogType::BranchSelector => {
                                if let Some(i) = state.dialog_state.selected() {
                                    let branch = state.available_branches[i].clone();
                                    state.branch_filter = Some(branch.clone());
                                    state.set_notification(format!(
                                        "Switched to branch: {}",
                                        branch
                                    ));
                                    state.show_dialog = None;
                                    state.files_state.select(Some(0));
                                    state.preview_dirty = true;
                                    state.dirty = true; // Fetch new file list
                                }
                            }
                            DialogType::Confirmation { .. } => {
                                state.show_dialog = None;
                            }
                        },
                        _ => {}
                    }
                    continue;
                }

                // --- Global Shortcuts (available in all views) ---
                match key_event.code {
                    KeyCode::Char('1') => {
                        state.view = ViewState::Home;
                        state.focus = Focus::Files;
                        state.dirty = true;
                    }
                    KeyCode::Char('2') => {
                        state.view = ViewState::Projects;
                        state.projects_state.select(Some(0));
                        state.dirty = true;
                    }
                    KeyCode::Char('3') => {
                        state.view = ViewState::Statistics;
                        state.dirty = true;
                    }
                    KeyCode::Char('/') => {
                        state.view = ViewState::Search;
                        state.input_mode = true;
                        state.search_query.clear();
                    }
                    KeyCode::Char('[') => {
                        state.sidebar_width_pct = state.sidebar_width_pct.saturating_sub(2).max(10);
                    }
                    KeyCode::Char(']') => {
                        state.sidebar_width_pct = state.sidebar_width_pct.saturating_add(2).min(70);
                    }
                    KeyCode::Char('{') => {
                        state.timeline_width_pct =
                            state.timeline_width_pct.saturating_sub(2).max(10);
                    }
                    KeyCode::Char('}') => {
                        state.timeline_width_pct =
                            state.timeline_width_pct.saturating_add(2).min(90);
                    }
                    _ => {}
                }

                // --- View-Specific Shortcuts ---
                match state.view {
                    ViewState::Home => match key_event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('f') => {
                            state.input_mode = true;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            state.view = ViewState::Settings;
                            state.settings_index = 0;
                            state.dirty = true;
                        }
                        KeyCode::Char('b') | KeyCode::Char('B') => {
                            if state.available_branches.is_empty() {
                                state.set_notification("No branches found".into());
                            } else {
                                state.show_dialog = Some(DialogType::BranchSelector);
                                state.dialog_state.select(Some(0));
                            }
                        }
                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                            if let Some(i) = state.files_state.selected() {
                                if let Some(f) = state.files.get(i) {
                                    state.selected_file = Some(f.path.clone());
                                    state.view = ViewState::History;
                                    state.focus = Focus::Timeline;
                                    state.versions_state.select(Some(0));
                                    state.preview_dirty = true;
                                    state.dirty = true;
                                    state.scroll_offset = 0;
                                    state.last_selection_time = std::time::Instant::now();
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            next(&mut state.files_state, state.files.len(), None);
                            // Auto-refresh history for the newly selected file
                            if let Some(i) = state.files_state.selected() {
                                if let Some(f) = state.files.get(i) {
                                    if let Ok(snapshots) = repo.get_history(&f.path) {
                                        state.history_items = group_snapshots(snapshots);
                                    }
                                }
                            }
                            state.preview_dirty = true;
                            state.scroll_offset = 0;
                            state.last_selection_time = std::time::Instant::now();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            prev(&mut state.files_state, state.files.len(), None);
                            // Auto-refresh history for the newly selected file
                            if let Some(i) = state.files_state.selected() {
                                if let Some(f) = state.files.get(i) {
                                    if let Ok(snapshots) = repo.get_history(&f.path) {
                                        state.history_items = group_snapshots(snapshots);
                                    }
                                }
                            }
                            state.preview_dirty = true;
                            state.scroll_offset = 0;
                            state.last_selection_time = std::time::Instant::now();
                        }
                        _ => {}
                    },
                    ViewState::History => match key_event.code {
                        KeyCode::Esc | KeyCode::Left | KeyCode::Backspace => match state.focus {
                            Focus::Preview => {
                                state.focus = Focus::Timeline;
                                state.preview_dirty = true;
                            }
                            _ => {
                                state.view = ViewState::Home;
                                state.focus = Focus::Files;
                                state.preview_dirty = true;
                                state.dirty = true;
                            }
                        },
                        KeyCode::Tab => {
                            state.focus = match state.focus {
                                Focus::Files => Focus::Timeline,
                                Focus::Timeline => Focus::Preview,
                                Focus::Preview => Focus::Files,
                            };
                            state.preview_dirty = true;
                            state.scroll_offset = 0;
                        }
                        KeyCode::BackTab => {
                            state.focus = match state.focus {
                                Focus::Files => Focus::Preview,
                                Focus::Timeline => Focus::Files,
                                Focus::Preview => Focus::Timeline,
                            };
                            state.preview_dirty = true;
                            state.scroll_offset = 0;
                        }
                        KeyCode::Char('p') | KeyCode::Char('P') => {
                            if state.focus != Focus::Preview {
                                state.focus = Focus::Preview;
                                state.preview_dirty = true;
                                state.scroll_offset = 0;
                                state.active_hunk_index = Some(1);
                            }
                        }
                        KeyCode::Char('r') => {
                            if let Some(file_path) = &state.selected_file {
                                if !state.selected_hunks.is_empty() {
                                    let mut curr: Option<String> = None;

                                    if let Some(i) = state.versions_state.selected() {
                                        if let Some(HistoryItem::Snapshot(s)) =
                                            state.history_items.get(i)
                                        {
                                            curr = Some(s.content_hash.clone());
                                        }
                                    }

                                    if let Some(snapshot_hash) = curr {
                                        let hunk_ids: Vec<usize> =
                                            state.selected_hunks.iter().cloned().collect();

                                        match repo.apply_selective_patch(
                                            file_path,
                                            &snapshot_hash,
                                            &hunk_ids,
                                        ) {
                                            Ok(_) => {
                                                state.set_notification(format!(
                                                    "Restored {} blocks successfully",
                                                    hunk_ids.len()
                                                ));
                                                state.selected_hunks.clear();
                                                state.preview_dirty = true;
                                            }
                                            Err(e) => {
                                                state.set_notification(format!(
                                                    "Restore failed: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                } else if let Some(idx) = state.versions_state.selected() {
                                    if let Some(HistoryItem::Snapshot(snapshot)) =
                                        state.history_items.get(idx)
                                    {
                                        let _ =
                                            repo.restore_file(&snapshot.content_hash, file_path);
                                        state.set_notification("Restored Full File".into());
                                    }
                                }
                            }
                        }
                        KeyCode::Char('v') => {
                            if let Some(idx) = state.versions_state.selected() {
                                if let Some(HistoryItem::Snapshot(snapshot)) =
                                    state.history_items.get(idx)
                                {
                                    if state.diff_base_hash.as_ref() == Some(&snapshot.content_hash)
                                    {
                                        state.diff_base_hash = None;
                                        state.set_notification("Diff base cleared".into());
                                    } else {
                                        state.diff_base_hash = Some(snapshot.content_hash.clone());
                                        state.set_notification("Diff base set".into());
                                    }
                                    state.preview_dirty = true;
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => match state.focus {
                            Focus::Preview => {
                                if state.hunk_count > 0 {
                                    let current = state.active_hunk_index.unwrap_or(0);
                                    state.active_hunk_index =
                                        Some((current % state.hunk_count) + 1);
                                    state.preview_dirty = true;
                                } else {
                                    state.scroll_offset = state.scroll_offset.saturating_add(1)
                                }
                            }
                            _ => {
                                next(
                                    &mut state.versions_state,
                                    state.history_items.len(),
                                    Some(&state.history_items),
                                );
                                state.preview_dirty = true;
                                state.scroll_offset = 0;
                                state.last_selection_time = std::time::Instant::now();
                                state.active_hunk_index = None;
                                state.selected_hunks.clear();
                            }
                        },
                        KeyCode::Up | KeyCode::Char('k') => match state.focus {
                            Focus::Preview => {
                                if state.hunk_count > 0 {
                                    let current = state.active_hunk_index.unwrap_or(1);
                                    state.active_hunk_index = Some(if current <= 1 {
                                        state.hunk_count
                                    } else {
                                        current - 1
                                    });
                                    state.preview_dirty = true;
                                } else {
                                    state.scroll_offset = state.scroll_offset.saturating_sub(1);
                                }
                            }
                            _ => {
                                prev(
                                    &mut state.versions_state,
                                    state.history_items.len(),
                                    Some(&state.history_items),
                                );
                                state.preview_dirty = true;
                                state.scroll_offset = 0;
                                state.last_selection_time = std::time::Instant::now();
                                state.active_hunk_index = None;
                                state.selected_hunks.clear();
                            }
                        },
                        KeyCode::Char(' ') => {
                            if state.focus == Focus::Preview || state.focus == Focus::Timeline {
                                state.last_selection_time =
                                    std::time::Instant::now() - Duration::from_millis(200);

                                if state.hunk_count > 0 {
                                    let idx = state.active_hunk_index.unwrap_or(1);
                                    state.active_hunk_index = Some(idx);
                                    if state.selected_hunks.contains(&idx) {
                                        state.selected_hunks.remove(&idx);
                                    } else {
                                        state.selected_hunks.insert(idx);
                                    }
                                    // Auto-move to next hunk for faster selection
                                    if state.hunk_count > 0 {
                                        state.active_hunk_index =
                                            Some((idx % state.hunk_count) + 1);
                                    }
                                    state.preview_dirty = true;
                                }
                            }
                        }
                        _ => {}
                    },
                    ViewState::Settings => match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            state.view = ViewState::Home;
                            state.dirty = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            state.settings_index = (state.settings_index + 1) % 8
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.settings_index = if state.settings_index == 0 {
                                7
                            } else {
                                state.settings_index - 1
                            }
                        }
                        KeyCode::Enter => {
                            let mut config_manager = match repo.config.lock() {
                                Ok(guard) => guard,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            match state.settings_index {
                                0 => {
                                    let days = if config_manager.config.retention_days == 30 {
                                        7
                                    } else {
                                        30
                                    };
                                    let _ = config_manager.update_retention(days);
                                    state.set_notification(format!(
                                        "Retention set to {} days",
                                        days
                                    ));
                                }
                                1 => {
                                    let _ = config_manager.toggle_compression();
                                    let status = if config_manager.config.compression_enabled {
                                        "Enabled"
                                    } else {
                                        "Disabled"
                                    };
                                    state.set_notification(format!("Compression {}", status));
                                }
                                2 => {
                                    config_manager.config.use_gitignore =
                                        !config_manager.config.use_gitignore;
                                    let _ = config_manager.save();
                                }
                                3 => {
                                    config_manager.config.use_mnemosyneignore =
                                        !config_manager.config.use_mnemosyneignore;
                                    let _ = config_manager.save();
                                }
                                4 => {
                                    let next_theme_idx =
                                        (config_manager.config.theme_index + 1) % THEMES.len();
                                    config_manager.config.theme_index = next_theme_idx;
                                    let _ = config_manager.save();
                                    state.theme = THEMES[next_theme_idx].clone();
                                }
                                5 => {
                                    use mnem_core::config::Ide;
                                    config_manager.config.ide = match config_manager.config.ide {
                                        Ide::Zed => Ide::ZedPreview,
                                        Ide::ZedPreview => Ide::VsCode,
                                        Ide::VsCode => Ide::Zed,
                                    };
                                    let _ = config_manager.save();
                                    state.set_notification(format!(
                                        "Primary IDE: {}",
                                        config_manager.config.ide.as_str()
                                    ));
                                }
                                6 => {
                                    // Run GC
                                    drop(config_manager);
                                    match repo.run_gc() {
                                        Ok(n) => state
                                            .set_notification(format!("GC Pruned {} snapshots", n)),
                                        Err(e) => {
                                            state.set_notification(format!("GC Error: {}", e))
                                        }
                                    }
                                }
                                7 => {
                                    // Clear All History
                                    drop(config_manager);
                                    match repo.clear_all_history() {
                                        Ok(n) => {
                                            state
                                                .set_notification(format!("Wiped {} snapshots", n));
                                            state.history_items.clear();
                                            state.files.clear();
                                            state.selected_file = None;
                                            state.preview_dirty = true;
                                        }
                                        Err(e) => {
                                            state.set_notification(format!("Wipe Error: {}", e))
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    ViewState::Projects => match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            state.view = ViewState::Home;
                            state.dirty = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            next(&mut state.projects_state, state.projects.len(), None);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            prev(&mut state.projects_state, state.projects.len(), None);
                        }
                        KeyCode::Char('d') => {
                            if let Some(idx) = state.projects_state.selected() {
                                if let Some(p) = state.projects.get(idx) {
                                    if let Ok(mut client) = DaemonClient::connect() {
                                        let _ = client.call(
                                            methods::PROJECT_UNWATCH,
                                            serde_json::json!({"project_path": p.project_path}),
                                        );
                                        state.set_notification(format!(
                                            "Unwatched {}",
                                            p.project_path
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    ViewState::Statistics => match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            state.view = ViewState::Home;
                            state.dirty = true;
                        }
                        KeyCode::Char('r') => {
                            state.stats = None; // Force refresh
                        }
                        _ => {}
                    },
                    ViewState::Search => match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            state.view = ViewState::Home;
                            state.dirty = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            next(&mut state.search_state, state.search_results.len(), None);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            prev(&mut state.search_state, state.search_results.len(), None);
                        }
                        KeyCode::Enter => {
                            if let Some(idx) = state.search_state.selected() {
                                if let Some(res) = state.search_results.get(idx) {
                                    state.selected_file = Some(res.file_path.clone());
                                    state.view = ViewState::History;
                                    state.focus = Focus::Timeline;
                                    state.versions_state.select(Some(0));
                                    state.preview_dirty = true;
                                }
                            }
                        }
                        _ => {}
                    },
                }
            }
            AppEvent::Tick => {
                // Reactive: Check if any new snapshots were added by mnemd in the background
                if let Ok(max_id) = repo.db.get_max_snapshot_id() {
                    if max_id > state.last_snapshot_id {
                        state.last_snapshot_id = max_id;
                        state.preview_dirty = true;
                        // Forcing re-fetch of history if we are in History view
                        if state.view == ViewState::History {
                            if let Some(path) = &state.selected_file {
                                if let Ok(snapshots) = repo.get_history(path) {
                                    state.history_items = group_snapshots(snapshots);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    Ok(())
}

fn next(state: &mut ratatui::widgets::ListState, len: usize, items: Option<&[HistoryItem]>) {
    if len == 0 {
        return;
    }
    let mut i = match state.selected() {
        Some(i) => (i + 1) % len,
        None => 0,
    };

    // Skip date headers if history items are provided
    if let Some(history) = items {
        let start = i;
        while matches!(history[i], HistoryItem::DateHeader(_)) {
            i = (i + 1) % len;
            if i == start {
                break;
            }
        }
    }
    state.select(Some(i));
}

fn prev(state: &mut ratatui::widgets::ListState, len: usize, items: Option<&[HistoryItem]>) {
    if len == 0 {
        return;
    }
    let mut i = match state.selected() {
        Some(i) => {
            if i == 0 {
                len - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };

    // Skip date headers if history items are provided
    if let Some(history) = items {
        let start = i;
        while matches!(history[i], HistoryItem::DateHeader(_)) {
            i = if i == 0 { len - 1 } else { i - 1 };
            if i == start {
                break;
            }
        }
    }
    state.select(Some(i));
}

fn group_snapshots(snapshots: Vec<Snapshot>) -> Vec<HistoryItem> {
    let mut items = Vec::new();
    if snapshots.is_empty() {
        return items;
    }

    let mut current_session = Vec::new();
    let mut last_processed: Option<Snapshot> = None;
    let mut last_date: Option<String> = None;

    for snap in snapshots {
        if let Ok(dt) = DateTime::parse_from_rfc3339(&snap.timestamp) {
            let date_str = dt.format("%A %d/%m").to_string();
            if last_date.as_ref() != Some(&date_str) {
                if !current_session.is_empty() {
                    push_session_group(&mut items, &current_session);
                    current_session.clear();
                }
                items.push(HistoryItem::DateHeader(date_str.clone()));
                last_date = Some(date_str);
            }
        }

        let is_new_session = if let Some(last) = &last_processed {
            if let (Ok(t1), Ok(t2)) = (
                DateTime::parse_from_rfc3339(&last.timestamp),
                DateTime::parse_from_rfc3339(&snap.timestamp),
            ) {
                let gap = t1.signed_duration_since(t2);
                gap.num_minutes() > 30 || last.git_branch != snap.git_branch
            } else {
                false
            }
        } else {
            false
        };

        if is_new_session && !current_session.is_empty() {
            push_session_group(&mut items, &current_session);
            current_session.clear();
        }

        current_session.push(snap.clone());
        last_processed = Some(snap);
    }
    push_session_group(&mut items, &current_session);
    items
}

fn push_session_group(items: &mut Vec<HistoryItem>, snaps: &[Snapshot]) {
    if snaps.is_empty() {
        return;
    }
    let newest = snaps.first().expect("snaps is not empty after check");

    let label = if let Ok(dt) = DateTime::parse_from_rfc3339(&newest.timestamp) {
        let time_of_day = match dt.hour() {
            5..=11 => "Morning",
            12..=17 => "Afternoon",
            18..=22 => "Evening",
            _ => "Night",
        };
        format!("{} Session", time_of_day)
    } else {
        "Unknown Session".to_string()
    };

    items.push(HistoryItem::Session(SessionInfo {
        label,
        branch: newest.git_branch.clone(),
        count: snaps.len(),
    }));

    for s in snaps {
        items.push(HistoryItem::Snapshot(s.clone()));
    }
}

fn update_preview(
    repo: &Repository,
    state: &mut AppState,
    ps: &SyntaxSet,
    theme_syntect: &syntect::highlighting::Theme,
    ts_highlighter: &mut TsHighlighter,
) -> Result<()> {
    if state.view == ViewState::Settings || state.view == ViewState::Search {
        state.cached_diff = vec![];
        return Ok(());
    }

    let (curr_hash, prev_hash) = match state.view {
        ViewState::Home => {
            if let Some(i) = state.files_state.selected() {
                if let Some(f) = state.files.get(i) {
                    let history = repo.get_history(&f.path)?;
                    (
                        history.first().map(|s| s.content_hash.clone()),
                        history.get(1).map(|s| s.content_hash.clone()),
                    )
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        }
        ViewState::History => {
            if let Some(i) = state.versions_state.selected() {
                match state.history_items.get(i) {
                    Some(HistoryItem::DateHeader(_)) => (None, None),
                    Some(HistoryItem::Snapshot(curr_snap)) => {
                        let curr = Some(curr_snap.content_hash.clone());
                        let mut prev = None;

                        if let Some(base) = &state.diff_base_hash {
                            prev = Some(base.clone());
                        } else if state.focus == Focus::Preview {
                            prev = Some("__DISK__".to_string());
                        } else {
                            for item in state.history_items.iter().skip(i + 1) {
                                match item {
                                    HistoryItem::Snapshot(s) => {
                                        prev = Some(s.content_hash.clone());
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        };
                        (curr, prev)
                    }
                    Some(HistoryItem::Session(_)) => {
                        let mut first = None;
                        let mut last = None;

                        for item in state.history_items.iter().skip(i + 1) {
                            match item {
                                HistoryItem::Snapshot(s) => {
                                    if last.is_none() {
                                        last = Some(s.content_hash.clone());
                                    }
                                    first = Some(s.content_hash.clone());
                                }
                                HistoryItem::Session(_) | HistoryItem::DateHeader(_) => break,
                            }
                        }
                        (last, first)
                    }
                    _ => (None, None),
                }
            } else {
                (None, None)
            }
        }
        _ => (None, None),
    };

    let cache_key = format!(
        "{:?}-{:?}-{:?}-{:?}-{:?}",
        state.view, curr_hash, prev_hash, state.active_hunk_index, state.selected_hunks
    );
    if state.last_diff_hash.as_ref() == Some(&cache_key) {
        return Ok(());
    }

    let mut lines = Vec::new();
    if let Some(ref c) = curr_hash {
        let mut fetch_error = None;
        let c_content = match repo.get_content(c) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(e) => {
                fetch_error = Some(e.to_string());
                String::new()
            }
        };

        if let Some(e) = fetch_error {
            lines.push(Line::from(vec![
                Span::styled(
                    " Error: ",
                    Style::default()
                        .bg(Color::Red)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" Failed to load content: {}", e),
                    Style::default().fg(Color::Red),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(
                " This snapshot may be corrupted or still being synchronized by the daemon.",
            ));
            state.cached_diff = lines;
            state.last_diff_hash = Some(cache_key);
            return Ok(());
        }

        let path = state.selected_file.as_deref().unwrap_or("txt");
        let syntax = ps
            .find_syntax_for_file(path)
            .unwrap_or(None)
            .unwrap_or_else(|| ps.find_syntax_plain_text());
        state.current_lang = Some(syntax.name.clone());

        let extension = path.split('.').last().unwrap_or("");
        let use_ts = extension == "rs" || extension == "rb";

        let highlight_content =
            |content: &str, state_ref: &AppState, ts_h: &mut TsHighlighter| -> Vec<Line<'static>> {
                if use_ts {
                    ts_h.highlight(content, extension, &state_ref.theme)
                } else {
                    let mut h = HighlightLines::new(syntax, theme_syntect);
                    content
                        .lines()
                        .map(|line| {
                            let ranges: Vec<(syntect::highlighting::Style, &str)> =
                                h.highlight_line(line, ps).unwrap_or_default();
                            let spans: Vec<Span> = ranges
                                .into_iter()
                                .map(|(style, text)| {
                                    let fg = style.foreground;
                                    Span::styled(
                                        text.to_string(),
                                        Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b)),
                                    )
                                })
                                .collect();
                            Line::from(spans)
                        })
                        .collect()
                }
            };

        if let Some(p) = prev_hash {
            let p_content = if p == "__DISK__" {
                if let Some(path) = &state.selected_file {
                    std::fs::read_to_string(path).unwrap_or_else(|_| String::new())
                } else {
                    String::new()
                }
            } else {
                match repo.get_content(&p) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => String::new(),
                }
            };

            let _p_highlighted = highlight_content(&p_content, state, ts_highlighter);
            let c_highlighted = highlight_content(&c_content, state, ts_highlighter);
            let diff = TextDiff::from_lines(&p_content, &c_content);

            let mut current_hunk = 0;
            let mut in_change = false;
            let mut plus = 0;
            let mut minus = 0;

            for change in diff.iter_all_changes() {
                let tag = change.tag();
                match tag {
                    ChangeTag::Insert => plus += 1,
                    ChangeTag::Delete => minus += 1,
                    _ => {}
                }

                if tag != ChangeTag::Equal {
                    if !in_change {
                        in_change = true;
                        current_hunk += 1;
                    }
                } else {
                    in_change = false;
                }

                let line_idx = change.new_index().or(change.old_index()).unwrap_or(0);
                let base_line = c_highlighted
                    .get(line_idx)
                    .cloned()
                    .unwrap_or_else(|| Line::from(""));
                let mut spans = base_line.spans;

                let prefix = match tag {
                    ChangeTag::Insert => "+ ",
                    ChangeTag::Delete => "- ",
                    ChangeTag::Equal => "  ",
                };

                let style = match tag {
                    ChangeTag::Insert => Style::default().fg(Color::Green),
                    ChangeTag::Delete => Style::default().fg(Color::Red),
                    ChangeTag::Equal => Style::default(),
                };

                spans.insert(0, Span::styled(prefix, style));

                let is_active = state.active_hunk_index == Some(current_hunk);
                let is_selected = state.selected_hunks.contains(&current_hunk);

                let line_style = if is_active {
                    Style::default().bg(Color::Rgb(60, 60, 60))
                } else if is_selected {
                    Style::default().bg(Color::Rgb(40, 40, 80))
                } else {
                    Style::default()
                };

                lines.push(Line::from(spans).style(line_style));
            }
            state.hunk_count = current_hunk;
            state.diff_plus = plus;
            state.diff_minus = minus;
        } else {
            lines = highlight_content(&c_content, state, ts_highlighter);
        }
    }
    state.cached_diff = lines;
    state.last_diff_hash = Some(cache_key);
    Ok(())
}
