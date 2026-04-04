use crate::app::{AppState, Focus};
use crate::components::shared::{ComponentFocus, ZedBlock};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(0),    // Results
        ])
        .split(area);

    // 1. Search Bar
    let search_block = ZedBlock::default(
        theme,
        " SEARCH ",
        if state.input_mode {
            ComponentFocus::Active
        } else {
            ComponentFocus::Inactive
        },
    );

    let (search_text, search_style) = if state.search_query.is_empty() && !state.input_mode {
        (
            "Press / to start searching...".to_string(),
            Style::default()
                .fg(theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        )
    } else if state.input_mode {
        // Show cursor character at end while in input mode
        (
            format!("{}_", state.search_query),
            Style::default().fg(theme.text_main),
        )
    } else {
        (
            state.search_query.clone(),
            Style::default().fg(theme.text_main),
        )
    };

    let p = Paragraph::new(Line::from(vec![
        Span::styled("   ", Style::default().fg(theme.accent)),
        Span::styled(search_text, search_style),
    ]))
    .block(search_block);

    f.render_widget(p, chunks[0]);

    // 2. Results
    let results_block = ZedBlock::default(
        theme,
        format!(" RESULTS ({}) ", state.search_results.len()),
        if state.focus == Focus::Files {
            ComponentFocus::Active
        } else {
            ComponentFocus::Inactive
        },
    );

    if state.search_results.is_empty() {
        let empty_msg = if state.search_query.is_empty() {
            "Enter a query above to search across all tracked files."
        } else {
            "No results found. Try a different search term."
        };
        let empty = Paragraph::new(Line::from(vec![Span::styled(
            empty_msg,
            Style::default()
                .fg(theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        )]))
        .block(results_block)
        .alignment(Alignment::Center);
        f.render_widget(empty, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = state
        .search_results
        .iter()
        .enumerate()
        .map(|(idx, res)| {
            let is_selected = state.search_state.selected() == Some(idx);

            let mut file_style = Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD);
            if is_selected {
                file_style = file_style.fg(theme.bg).bg(theme.accent);
            }

            let content_preview = res.content.trim();

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", res.file_path), file_style),
                    Span::styled(
                        format!(" :{}", res.line_number),
                        Style::default().fg(theme.text_dim),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(content_preview, Style::default().fg(theme.text_main)),
                ]),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(results_block)
        .highlight_style(Style::default().bg(theme.sidebar));

    f.render_stateful_widget(list, chunks[1], &mut state.search_state);
}
