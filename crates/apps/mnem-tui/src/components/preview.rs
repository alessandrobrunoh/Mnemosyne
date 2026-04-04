use crate::app::{AppState, Focus};
use crate::components::shared::{ComponentFocus, ZedBlock};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Wrap},
};

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;
    let is_focused = state.focus == Focus::Preview;

    use ratatui::text::{Line, Span};

    let lang = state.current_lang.as_deref().unwrap_or("Plain Text");

    let total_lines = state.cached_diff.len();
    let scroll_start_line = state.scroll_offset as usize;

    let mut title_spans = vec![
        Span::raw(" DIFF — "),
        Span::styled(
            lang,
            if is_focused {
                ratatui::style::Style::default()
                    .fg(theme.accent)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                ratatui::style::Style::default().fg(theme.text_main)
            },
        ),
    ];

    if state.diff_plus > 0 {
        title_spans.push(Span::styled(
            format!(" +{} ", state.diff_plus),
            ratatui::style::Style::default()
                .fg(theme.diff_add_fg)
                .bg(theme.diff_add_bg),
        ));
    }
    if state.diff_minus > 0 {
        title_spans.push(Span::styled(
            format!(" -{} ", state.diff_minus),
            ratatui::style::Style::default()
                .fg(theme.diff_del_fg)
                .bg(theme.diff_del_bg),
        ));
    }

    if total_lines > 0 {
        title_spans.push(Span::styled(
            format!(" {}:{} ", scroll_start_line + 1, total_lines),
            ratatui::style::Style::default().fg(theme.text_dim),
        ));
    } else {
        title_spans.push(Span::raw(" "));
    }

    let block = ZedBlock::default(
        theme,
        Line::from(title_spans),
        if is_focused {
            ComponentFocus::Active
        } else {
            ComponentFocus::Inactive
        },
    );

    let scroll = (state.scroll_offset.min(u16::MAX as u32) as u16, 0);

    if state.cached_diff.is_empty() {
        let empty = Paragraph::new(Line::from(vec![Span::styled(
            "No diff available. Select a snapshot from the timeline.",
            ratatui::style::Style::default().fg(theme.text_dim),
        )]))
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty, area);
    } else {
        let preview = Paragraph::new(state.cached_diff.clone())
            .block(block)
            .scroll(scroll)
            .wrap(Wrap { trim: false });
        f.render_widget(preview, area);
    }
}
