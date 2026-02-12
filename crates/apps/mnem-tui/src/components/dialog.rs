use crate::app::{AppState, DialogType};
use crate::components::shared::ZedBlock;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, state: &mut AppState) {
    let dialog_type = match &state.show_dialog {
        Some(t) => t,
        None => return,
    };

    let area = centered_rect(60, 40, f.size());
    f.render_widget(Clear, area);

    match dialog_type {
        DialogType::BranchSelector => render_branch_selector(f, area, state),
        DialogType::Confirmation { title, message } => {
            render_confirmation(f, area, state, title, message)
        }
    }
}

fn render_branch_selector(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;
    let block = ZedBlock::default(
        theme,
        "SWITCH BRANCH",
        crate::components::shared::ComponentFocus::Active,
    );

    let items: Vec<ListItem> = state
        .available_branches
        .iter()
        .map(|b| {
            let is_current = Some(b) == state.branch_filter.as_ref();
            let content = if is_current {
                Line::from(vec![
                    Span::styled(" • ", Style::default().fg(theme.accent)),
                    Span::styled(
                        b.to_string(),
                        Style::default()
                            .fg(theme.text_main)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(b.to_string(), Style::default().fg(theme.text_main)),
                ])
            };
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(theme.surface).fg(theme.accent))
        .highlight_symbol(" > ");

    f.render_stateful_widget(list, area, &mut state.dialog_state);
}

fn render_confirmation(f: &mut Frame, area: Rect, state: &AppState, title: &str, message: &str) {
    let theme = &state.theme;
    let block = ZedBlock::default(
        theme,
        title,
        crate::components::shared::ComponentFocus::Active,
    );

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            message,
            Style::default().fg(theme.text_main),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [Enter] Confirm ",
                Style::default().bg(theme.accent).fg(theme.bg).bold(),
            ),
            Span::styled("   ", Style::default()),
            Span::styled(" [Esc] Cancel ", Style::default().fg(theme.text_dim)),
        ])
        .alignment(Alignment::Center),
    ])
    .block(block)
    .alignment(Alignment::Center);

    f.render_widget(text, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
