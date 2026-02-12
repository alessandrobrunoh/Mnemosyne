use crate::app::{AppState, ViewState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(15), // Logo
            Constraint::Min(0),     // Tabs / Breadcrumbs
            Constraint::Length(25), // Branch info
        ])
        .split(area);

    // 1. Logo
    let logo = Paragraph::new(Span::styled(
        " MNEMOSYNE ",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))
    .bg(theme.bg);
    f.render_widget(logo, chunks[0]);

    // 2. Tabs / Breadcrumbs
    let mut header_items = vec![];

    // Navigation Tabs
    let tabs = vec![
        (ViewState::Home, " 󱂵 HOME (1) "),
        (ViewState::Search, "  SEARCH (/) "),
        (ViewState::Projects, " 󱔗 PROJECTS (2) "),
        (ViewState::Statistics, " 󰄳 STATS (3) "),
        (ViewState::Settings, "  SETTINGS (S) "),
    ];

    for (view, label) in tabs {
        let is_active = state.view == view;
        let style = if is_active {
            Style::default().fg(theme.bg).bg(theme.accent).bold()
        } else {
            Style::default().fg(theme.text_dim)
        };
        header_items.push(Span::styled(label, style));
        header_items.push(Span::styled(" ", Style::default()));
    }

    header_items.push(Span::styled(" │ ", Style::default().fg(theme.text_dim)));

    // Project context
    header_items.push(Span::styled(
        format!(" {} ", state.project_name),
        Style::default().fg(theme.accent).bold(),
    ));

    if let Some(ref path) = state.selected_file {
        header_items.push(Span::styled(" › ", Style::default().fg(theme.text_dim)));

        let path_parts: Vec<&str> = path.split('/').collect();
        let total_parts = path_parts.len();

        for (i, part) in path_parts.iter().enumerate() {
            let is_last = i == total_parts - 1;
            let style = if is_last {
                Style::default()
                    .fg(theme.text_main)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_dim)
            };

            header_items.push(Span::styled(*part, style));
            if !is_last {
                header_items.push(Span::styled("/", Style::default().fg(theme.text_dim)));
            }
        }
    }

    let bc_para = Paragraph::new(Line::from(header_items)).bg(theme.bg);
    f.render_widget(bc_para, chunks[1]);

    // 3. Branch Indicator
    if let Some(ref branch) = state.git_branch {
        let branch_info = Paragraph::new(Line::from(vec![
            Span::styled(" on ", Style::default().fg(theme.text_dim)),
            Span::styled(" ", Style::default().fg(theme.accent)),
            Span::styled(
                branch,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" [x!?]", Style::default().fg(Color::Rgb(224, 108, 117))), // GitButler red status
        ]))
        .alignment(ratatui::layout::Alignment::Right)
        .bg(theme.bg);
        f.render_widget(branch_info, chunks[2]);
    }
}
