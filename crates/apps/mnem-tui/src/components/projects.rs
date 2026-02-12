use crate::app::AppState;
use crate::components::shared::{ComponentFocus, ZedBlock};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/Action info
            Constraint::Min(0),    // Project List
        ])
        .split(area);

    // 1. Info / Instructions
    let info_block = ZedBlock::default(theme, " PROJECTS ", ComponentFocus::Inactive);
    let p = Paragraph::new(Line::from(vec![
        Span::styled(" 󱔗 ", Style::default().fg(theme.accent)),
        Span::styled(
            "Manage watched projects. Press ",
            Style::default().fg(theme.text_main),
        ),
        Span::styled(
            "'a'",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to add, ", Style::default().fg(theme.text_main)),
        Span::styled(
            "'d'",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to remove.", Style::default().fg(theme.text_main)),
    ]))
    .block(info_block);
    f.render_widget(p, chunks[0]);

    // 2. Project List
    let list_block = ZedBlock::default(
        theme,
        format!(" WATCHED PROJECTS ({}) ", state.projects.len()),
        ComponentFocus::Active,
    );

    let items: Vec<ListItem> = state
        .projects
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let is_selected = state.projects_state.selected() == Some(idx);

            let mut name_style = Style::default()
                .fg(theme.text_main)
                .add_modifier(Modifier::BOLD);
            if is_selected {
                name_style = name_style.fg(theme.bg).bg(theme.accent);
            }

            ListItem::new(vec![
                Line::from(vec![Span::styled(
                    format!(" {} ", p.project_path),
                    name_style,
                )]),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(
                        format!(
                            "󰈔 {} files |  {} snapshots",
                            p.file_count, p.snapshot_count
                        ),
                        Style::default().fg(theme.text_dim),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(
                        format!("Last activity: {}", p.last_activity),
                        Style::default().fg(theme.text_dim),
                    ),
                ]),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().bg(theme.sidebar));

    f.render_stateful_widget(list, chunks[1], &mut state.projects_state);
}
