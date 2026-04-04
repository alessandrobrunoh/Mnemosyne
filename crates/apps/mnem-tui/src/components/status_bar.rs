use crate::app::{AppState, Focus, ViewState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let help_text = match state.view {
        ViewState::Home | ViewState::History => {
            if state.input_mode {
                " [Esc] Stop Filtering  [Enter] Confirm ".to_string()
            } else {
                match state.focus {
                    Focus::Files => {
                        " [↑↓] Navigate  [Enter] Open  [F] Filter  [B] Branch  [Tab] Focus  [Q] Quit ".to_string()
                    }
                    Focus::Timeline => {
                        " [↑↓] Navigate  [V] Compare  [Space] Select Hunk  [Tab] Focus  [Esc] Back ".to_string()
                    }
                    Focus::Preview => {
                        let r_label = if state.selected_hunks.is_empty() {
                            "Restore File"
                        } else {
                            "Restore Hunks"
                        };
                        format!(
                            " [↑↓] Scroll  [Space] Select Hunk  [R] {}  [Tab] Focus  [Esc] Back ",
                            r_label
                        )
                    }
                }
            }
        }
        ViewState::Search => {
            if state.input_mode {
                " [Esc] Stop Searching  [Enter] Confirm ".to_string()
            } else {
                " [/] Search  [↑↓] Navigate  [Enter] View  [Esc] Back ".to_string()
            }
        }
        ViewState::Projects => " [↑↓] Navigate  [D] Unwatch  [Esc] Back ".to_string(),
        ViewState::Statistics => " [R] Refresh  [Esc] Back ".to_string(),
        ViewState::Settings => " [↑↓] Navigate  [Enter] Change  [Esc] Back ".to_string(),
    };

    let help_text_cow = std::borrow::Cow::from(help_text);

    let (mode_label, mode_bg) = if state.input_mode {
        // Use theme success color for INSERT mode to clearly differentiate from NORMAL
        (" INSERT ", theme.success)
    } else {
        (" NORMAL ", theme.accent)
    };

    let mode_width = mode_label.len() as u16;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(mode_width)])
        .split(area);

    let shortcuts = Paragraph::new(Line::from(vec![Span::styled(
        help_text_cow,
        Style::default().fg(theme.text_dim),
    )]))
    .bg(theme.sidebar);

    f.render_widget(shortcuts, chunks[0]);

    let mode = Paragraph::new(Line::from(vec![Span::styled(
        mode_label,
        Style::default()
            .bg(mode_bg)
            .fg(theme.bg)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Right)
    .bg(theme.sidebar);

    f.render_widget(mode, chunks[1]);
}
