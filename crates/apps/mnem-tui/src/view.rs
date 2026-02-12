use crate::app::{AppState, ViewState};
use crate::components::{
    header, preview, projects, search, settings, shared::ZedBlock, sidebar, stats, status_bar,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::Clear,
    Frame,
};

pub fn render(f: &mut Frame, state: &mut AppState) {
    state.clear_expired_notifications();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Header (Breadcrumbs/Tabs)
            Constraint::Min(0),    // Main Body
            Constraint::Length(1), // Bottom Status Bar
        ])
        .split(f.size());

    // 1. Render Background
    f.render_widget(ZedBlock::ghost(&state.theme), f.size());

    // 2. Header
    header::render(f, chunks[0], state);

    // 3. Main Content Area
    match state.view {
        ViewState::Settings => settings::render(f, chunks[1], state),
        ViewState::Search => search::render(f, chunks[1], state),
        ViewState::Projects => projects::render(f, chunks[1], state),
        ViewState::Statistics => stats::render(f, chunks[1], state),
        _ => {
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25), // FILES
                    Constraint::Percentage(25), // SNAPSHOTS (Timeline)
                    Constraint::Percentage(50), // DIFF (Preview)
                ])
                .split(chunks[1]);

            sidebar::render_files(f, body_chunks[0], state);
            sidebar::render_timeline(f, body_chunks[1], state);
            preview::render(f, body_chunks[2], state);
        }
    }

    // 4. Status Bar
    status_bar::render(f, chunks[2], state);

    // 5. Dialogs (Popups)
    crate::components::dialog::render(f, state);

    // 6. Toasts / Notifications
    if let Some((msg, _)) = &state.notification {
        render_toast(f, msg, state);
    }
}

// View logic handled by components

fn render_toast(f: &mut Frame, message: &str, state: &crate::app::AppState) {
    use ratatui::{
        layout::Alignment,
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };
    let area = centered_rect(50, 10, f.size());
    let toast_area = ratatui::layout::Rect::new(area.x, f.size().height - 4, area.width, 3);
    let text = Paragraph::new(Line::from(vec![
        Span::styled(
            " INFO ",
            Style::default()
                .bg(state.theme.accent)
                .fg(state.theme.bg)
                .bold(),
        ),
        Span::styled(
            format!(" {} ", message),
            Style::default().fg(state.theme.text_main),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(state.theme.accent))
            .bg(state.theme.sidebar),
    )
    .alignment(Alignment::Center);
    f.render_widget(Clear, toast_area);
    f.render_widget(text, toast_area);
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
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
