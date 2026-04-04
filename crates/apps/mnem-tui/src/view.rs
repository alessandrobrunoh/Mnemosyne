use crate::app::{AppState, ViewState};
use crate::components::{
    header, preview, projects, search, settings, shared::ZedBlock, sidebar, stats, status_bar,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Clear,
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

    // Extra horizontal padding inside the toast (icon + spaces on each side)
    const TOAST_PADDING: u16 = 16;
    // Vertical offset from the bottom of the frame (height + 1 for the border row)
    const TOAST_BOTTOM_OFFSET: u16 = 4;
    // Minimum frame margin to keep the toast from touching screen edges
    const TOAST_FRAME_MARGIN: u16 = 4;
    // Height of the toast widget (content line + top/bottom border)
    const TOAST_HEIGHT: u16 = 3;

    let frame_width = f.size().width;
    let toast_width = (message.len() as u16 + TOAST_PADDING)
        .min(frame_width.saturating_sub(TOAST_FRAME_MARGIN));
    let toast_x = (frame_width.saturating_sub(toast_width)) / 2;
    let toast_area = ratatui::layout::Rect::new(
        toast_x,
        f.size().height.saturating_sub(TOAST_BOTTOM_OFFSET),
        toast_width,
        TOAST_HEIGHT,
    );
    let text = Paragraph::new(Line::from(vec![
        Span::styled(
            " 󰍡 ",
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
    .alignment(Alignment::Left);
    f.render_widget(Clear, toast_area);
    f.render_widget(text, toast_area);
}
