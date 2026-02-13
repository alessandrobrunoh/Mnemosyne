use crate::app::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    // Layout principale: Header + Body + Status
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Thin header line
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status line
        ])
        .split(area);

    // 1. HEADER - Linea sottile con tab e info contestuali
    render_header(f, main_chunks[0], state);

    // 2. MAIN CONTENT - Area principale
    render_main_content(f, main_chunks[1], state);

    // 3. STATUS BAR - Linea in basso con shortcuts
    render_status_bar(f, main_chunks[2], state);
}

fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Logo
            Constraint::Min(0),     // Tabs
            Constraint::Length(30), // Context info
        ])
        .split(area);

    // Logo
    let logo = Paragraph::new("mnemosyne")
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left);
    f.render_widget(logo, header_chunks[0]);

    // Tabs
    let tabs = vec![
        (" 󱂵 ", "files", ViewState::Home),
        (" 󰋚 ", "history", ViewState::History),
        ("  ", "search", ViewState::Search),
        (" 󱔗 ", "projects", ViewState::Projects),
        (" 󰄳 ", "stats", ViewState::Statistics),
        ("  ", "settings", ViewState::Settings),
    ];

    let tab_titles: Vec<Line> = tabs
        .iter()
        .map(|(icon, name, view)| {
            let is_active = state.view == *view;
            let style = if is_active {
                Style::default()
                    .fg(theme.bg)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_dim)
            };
            Line::from(vec![Span::styled(*icon, style), Span::styled(*name, style)])
        })
        .collect();

    let tabs_widget = Tabs::new(tab_titles)
        .select(match state.view {
            ViewState::Home => 0,
            ViewState::History => 1,
            ViewState::Search => 2,
            ViewState::Projects => 3,
            ViewState::Statistics => 4,
            ViewState::Settings => 5,
        })
        .style(Style::default().fg(theme.text_dim))
        .highlight_style(Style::default().fg(theme.bg).bg(theme.accent));

    f.render_widget(tabs_widget, header_chunks[1]);

    // Context (project/branch)
    let context_text = if let Some(ref branch) = state.git_branch {
        format!("{}  {}", state.project_name, branch)
    } else {
        state.project_name.clone()
    };

    let context = Paragraph::new(context_text)
        .style(Style::default().fg(theme.text_dim))
        .alignment(Alignment::Right);
    f.render_widget(context, header_chunks[2]);
}

fn render_main_content(f: &mut Frame, area: Rect, state: &mut AppState) {
    match state.view {
        ViewState::Home => render_home_view(f, area, state),
        ViewState::History => render_history_view(f, area, state),
        ViewState::Search => render_search_view(f, area, state),
        ViewState::Projects => render_projects_view(f, area, state),
        ViewState::Statistics => render_stats_view(f, area, state),
        ViewState::Settings => render_settings_view(f, area, state),
    }
}

fn render_home_view(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;

    // Split: Sidebar (files) | Content (preview/diff)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // File list
            Constraint::Percentage(70), // Preview
        ])
        .split(area);

    // File List Panel
    let file_list = List::new(state.files.iter().enumerate().map(|(i, file)| {
        let is_selected = state.files_state.selected() == Some(i);
        let style = if is_selected {
            Style::default()
                .bg(theme.surface)
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_main)
        };

        let icon = if file.path.ends_with('/') {
            "󰉋 "
        } else {
            "󰈙 "
        };
        let name = Path::new(&file.path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        ListItem::new(Line::from(vec![
            Span::styled(icon, Style::default().fg(theme.accent)),
            Span::styled(name.to_string(), style),
        ]))
    }))
    .block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(theme.border),
    )
    .highlight_style(Style::default().bg(theme.surface));

    f.render_stateful_widget(file_list, chunks[0], &mut state.files_state);

    // Preview Panel
    if let Some(i) = state.files_state.selected() {
        if let Some(file) = state.files.get(i) {
            let preview = render_file_preview(file, state);
            f.render_widget(preview, chunks[1]);
        }
    } else {
        let empty = Paragraph::new("Select a file to preview")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        f.render_widget(empty, chunks[1]);
    }
}

fn render_history_view(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;

    // Layout: Timeline | Diff
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Timeline
            Constraint::Percentage(75), // Diff
        ])
        .split(area);

    // Timeline Panel
    let items: Vec<ListItem> = state
        .history_items
        .iter()
        .map(|item| match item {
            HistoryItem::DateHeader(date) => ListItem::new(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    date.clone(),
                    Style::default()
                        .fg(theme.text_dim)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .style(Style::default().bg(theme.surface)),

            HistoryItem::Session(session) => ListItem::new(Line::from(vec![
                Span::styled("  ├─ ", Style::default().fg(theme.border)),
                Span::styled(&session.label, Style::default().fg(theme.accent)),
                Span::styled(
                    format!(" ({})", session.count),
                    Style::default().fg(theme.text_dim),
                ),
            ])),

            HistoryItem::Snapshot(snap) => {
                let hash = &snap.content_hash[..8];
                let time = format_time(&snap.timestamp);
                ListItem::new(Line::from(vec![
                    Span::styled("  │  ", Style::default().fg(theme.border)),
                    Span::styled("● ", Style::default().fg(theme.success)),
                    Span::styled(hash, Style::default().fg(theme.accent)),
                    Span::styled(" ", Style::default()),
                    Span::styled(time, Style::default().fg(theme.text_dim)),
                ]))
            }
        })
        .collect();

    let timeline = List::new(items)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(theme.border),
        )
        .highlight_style(Style::default().bg(theme.surface));

    f.render_stateful_widget(timeline, chunks[0], &mut state.versions_state);

    // Diff Panel
    let diff_widget = Paragraph::new(state.cached_diff.clone())
        .block(
            Block::default()
                .title(" Diff ")
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .scroll((state.scroll_offset as u16, 0));

    f.render_widget(diff_widget, chunks[1]);
}

fn render_search_view(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Results
        ])
        .split(area);

    // Search Input
    let input = Paragraph::new(state.search_query.clone())
        .block(
            Block::default()
                .title(" Search ")
                .borders(Borders::ALL)
                .border_style(if state.input_mode {
                    theme.accent
                } else {
                    theme.border
                }),
        )
        .style(Style::default().fg(theme.text_main));
    f.render_widget(input, chunks[0]);

    // Results
    let results: Vec<ListItem> = state
        .search_results
        .iter()
        .map(|res| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        &res.file_path,
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(":{}", res.line_number),
                        Style::default().fg(theme.text_dim),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&res.content, Style::default().fg(theme.text_main)),
                ]),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(results)
        .block(
            Block::default()
                .title(format!(" Results ({}) ", state.search_results.len()))
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .highlight_style(Style::default().bg(theme.surface));

    f.render_stateful_widget(list, chunks[1], &mut state.search_state);
}

fn render_projects_view(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;

    let items: Vec<ListItem> = state
        .projects
        .iter()
        .map(|p| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled("󰉋 ", Style::default().fg(theme.accent)),
                    Span::styled(
                        &p.project_path,
                        Style::default()
                            .fg(theme.text_main)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(
                        format!("{} files", p.file_count),
                        Style::default().fg(theme.text_dim),
                    ),
                    Span::styled(" • ", Style::default().fg(theme.border)),
                    Span::styled(
                        format!("{} snapshots", p.snapshot_count),
                        Style::default().fg(theme.text_dim),
                    ),
                    Span::styled(" • ", Style::default().fg(theme.border)),
                    Span::styled(&p.last_activity, Style::default().fg(theme.text_dim)),
                ]),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Watched Projects ")
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .highlight_style(Style::default().bg(theme.surface));

    f.render_stateful_widget(list, area, &mut state.projects_state);
}

fn render_stats_view(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;

    if let Some(stats) = &state.stats {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // Overview row
                Constraint::Length(10), // Chart
                Constraint::Min(0),     // Details
            ])
            .split(area);

        // Overview Cards
        let overview = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(chunks[0]);

        render_stat_card(
            f,
            overview[0],
            "Snapshots",
            &stats.total_snapshots.to_string(),
            theme,
        );
        render_stat_card(
            f,
            overview[1],
            "Files",
            &stats.total_files.to_string(),
            theme,
        );
        render_stat_card(
            f,
            overview[2],
            "Size",
            &format!("{:.1} MB", stats.size_bytes as f64 / 1_048_576.0),
            theme,
        );
        render_stat_card(f, overview[3], "Activity", &stats.last_activity, theme);

        // Activity Chart
        let activity_data: Vec<u64> = stats
            .activity_by_day
            .iter()
            .map(|(_, c)| *c as u64)
            .collect();
        let chart = Sparkline::default()
            .data(&activity_data)
            .style(Style::default().fg(theme.accent))
            .block(
                Block::default()
                    .title(" Activity (30d) ")
                    .borders(Borders::ALL)
                    .border_style(theme.border),
            );
        f.render_widget(chart, chunks[1]);

        // Details
        let details = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        // Top Files
        let top_files = stats
            .top_files
            .iter()
            .map(|(f, c)| {
                Line::from(vec![
                    Span::styled(format!("{:>4}", c), Style::default().fg(theme.accent)),
                    Span::styled(" ", Style::default()),
                    Span::styled(f, Style::default().fg(theme.text_main)),
                ])
            })
            .collect::<Vec<_>>();

        let top_files_widget = Paragraph::new(top_files).block(
            Block::default()
                .title(" Top Files ")
                .borders(Borders::ALL)
                .border_style(theme.border),
        );
        f.render_widget(top_files_widget, details[0]);

        // Extensions
        let exts = stats
            .extensions
            .iter()
            .map(|(e, c)| {
                Line::from(vec![
                    Span::styled(format!("{:>4}", c), Style::default().fg(theme.success)),
                    Span::styled(" ", Style::default()),
                    Span::styled(e, Style::default().fg(theme.text_main)),
                ])
            })
            .collect::<Vec<_>>();

        let exts_widget = Paragraph::new(exts).block(
            Block::default()
                .title(" Extensions ")
                .borders(Borders::ALL)
                .border_style(theme.border),
        );
        f.render_widget(exts_widget, details[1]);
    } else {
        let loading = Paragraph::new("Loading statistics...")
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center);
        f.render_widget(loading, area);
    }
}

fn render_settings_view(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let items = vec![
        ("Retention", format!("{} days", state.config.retention_days)),
        (
            "Compression",
            if state.config.compression_enabled {
                "On"
            } else {
                "Off"
            }
            .to_string(),
        ),
        ("Theme", THEMES[state.config.theme_index].name.clone()),
        ("IDE", state.config.ide.as_str().to_string()),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (k, v))| {
            let is_selected = i == state.settings_index;
            let style = if is_selected {
                Style::default()
                    .bg(theme.surface)
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_main)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:20}", k), style),
                Span::styled(v.clone(), Style::default().fg(theme.text_dim)),
            ]))
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(" Settings ")
                .borders(Borders::ALL)
                .border_style(theme.border),
        )
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let shortcuts = match state.view {
        ViewState::Home => " q:quit  enter:history  f:filter  b:branch  [/]:resize ",
        ViewState::History => " q:back  space:select-hunk  r:restore  v:compare  p:preview ",
        ViewState::Search => " enter:open  esc:back ",
        ViewState::Projects => " d:unwatch  esc:back ",
        ViewState::Statistics => " r:refresh  esc:back ",
        ViewState::Settings => " enter:change  esc:back ",
    };

    let help = Paragraph::new(shortcuts)
        .style(Style::default().fg(theme.text_dim).bg(theme.surface))
        .alignment(Alignment::Left);
    f.render_widget(help, area);
}

fn render_stat_card(f: &mut Frame, area: Rect, title: &str, value: &str, theme: &Theme) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.border);

    let text = Paragraph::new(value)
        .block(block)
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}
