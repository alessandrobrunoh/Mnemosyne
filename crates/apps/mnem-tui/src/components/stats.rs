use crate::app::AppState;
use crate::components::shared::ZedBlock;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = &state.theme;
    let stats = match &state.stats {
        Some(s) => s,
        None => {
            let p = Paragraph::new("Loading statistics...").block(ZedBlock::default(
                theme,
                " STATISTICS ",
                crate::components::shared::ComponentFocus::Inactive,
            ));
            f.render_widget(p, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Overview
            Constraint::Length(10), // Activity Sparkline
            Constraint::Min(0),     // Details
        ])
        .split(area);

    // 1. Overview
    let overview_block = ZedBlock::default(
        theme,
        " OVERVIEW ",
        crate::components::shared::ComponentFocus::Inactive,
    );
    let size_mb = stats.size_bytes as f64 / 1024.0 / 1024.0;

    let overview_text = vec![
        Line::from(vec![
            Span::styled(" Total Snapshots: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{}", stats.total_snapshots),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Total Files:     ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{}", stats.total_files),
                Style::default().fg(theme.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Total Branches:  ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{}", stats.total_branches),
                Style::default().fg(theme.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Database Size:   ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{:.2} MB", size_mb),
                Style::default().fg(theme.success),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Last Activity:   ", Style::default().fg(theme.text_dim)),
            Span::styled(&stats.last_activity, Style::default().fg(theme.text_main)),
        ]),
    ];
    let p = Paragraph::new(overview_text).block(overview_block);
    f.render_widget(p, chunks[0]);

    // 2. Activity Sparkline
    let activity_data: Vec<u64> = stats
        .activity_by_day
        .iter()
        .map(|(_, count)| *count as u64)
        .collect();
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ACTIVITY (Last 30 Days) ")
                .border_style(Style::default().fg(theme.text_dim)),
        )
        .data(&activity_data)
        .style(Style::default().fg(theme.accent));
    f.render_widget(sparkline, chunks[1]);

    // 3. Details (Top Files / Extensions)
    let details_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Top Files
    let top_files_block = Block::default()
        .borders(Borders::ALL)
        .title(" TOP FILES ")
        .border_style(Style::default().fg(theme.text_dim));
    let top_files_lines: Vec<Line> = stats
        .top_files
        .iter()
        .map(|(path, count)| {
            Line::from(vec![
                Span::styled(format!(" {} ", count), Style::default().fg(theme.accent)),
                Span::styled(path, Style::default().fg(theme.text_main)),
            ])
        })
        .collect();
    f.render_widget(
        Paragraph::new(top_files_lines).block(top_files_block),
        details_chunks[0],
    );

    // Extensions
    let ext_block = Block::default()
        .borders(Borders::ALL)
        .title(" EXTENSIONS ")
        .border_style(Style::default().fg(theme.text_dim));
    let ext_lines: Vec<Line> = stats
        .extensions
        .iter()
        .map(|(ext, count)| {
            Line::from(vec![
                Span::styled(format!(" {} ", count), Style::default().fg(theme.success)),
                Span::styled(ext, Style::default().fg(theme.text_main)),
            ])
        })
        .collect();
    f.render_widget(
        Paragraph::new(ext_lines).block(ext_block),
        details_chunks[1],
    );
}
