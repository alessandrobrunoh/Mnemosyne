use crate::app::{AppState, Focus};
use crate::components::shared::{ComponentFocus, ZedBlock};
use ratatui::{
    layout::Rect,
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;
    let is_focused = state.focus == Focus::Preview;

    use ratatui::text::{Line, Span};

    let lang = state.current_lang.as_deref().unwrap_or("Plain Text");

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
    title_spans.push(Span::raw(" "));

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

    let preview = Paragraph::new(state.cached_diff.clone())
        .block(block)
        .scroll(scroll)
        .wrap(Wrap { trim: false });

    f.render_widget(preview, area);
}
