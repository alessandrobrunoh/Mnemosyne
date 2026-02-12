use crate::app::{AppState, Focus, ViewState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let help_text = match state.view {
        ViewState::Home | ViewState::History => match state.focus {
            Focus::Files => {
                " [Ent] Open  [F] Filter  [B] Branch  [[/]] Resize  [Q] Quit ".to_string()
            }
            Focus::Timeline => " [P] Preview  [V] Compare  [[/]] Resize  [Esc] Back ".to_string(),
            Focus::Preview => {
                let r_label = if state.selected_hunks.is_empty() {
                    "Restore File"
                } else {
                    "Restore Hunks"
                };
                format!(
                    " [Space] Select Hunk  [R] {}  [[/]] Resize  [Esc] Back ",
                    r_label
                )
            }
        },
        ViewState::Search => " [Ent] View  [Esc] Back ".to_string(),
        ViewState::Projects => " [D] Unwatch  [Esc] Back ".to_string(),
        ViewState::Statistics => " [R] Refresh  [Esc] Back ".to_string(),
        ViewState::Settings => " [Ent] Change  [Esc] Back ".to_string(),
    };

    let help_text_cow = std::borrow::Cow::from(help_text);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(area);

    let shortcuts = Paragraph::new(Line::from(vec![Span::styled(
        help_text_cow,
        Style::default().fg(theme.text_dim),
    )]))
    .bg(theme.sidebar);

    f.render_widget(shortcuts, chunks[0]);

    let mode = Paragraph::new(Line::from(vec![Span::styled(
        " NORMAL ",
        Style::default()
            .bg(theme.accent)
            .fg(theme.bg)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Right)
    .bg(theme.sidebar);

    f.render_widget(mode, chunks[1]);
}
