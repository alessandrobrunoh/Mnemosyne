use crate::app::AppState;
use crate::components::shared::{ComponentFocus, ZedBlock};
use crate::theme::THEMES;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = &state.theme;

    let block = ZedBlock::default(theme, "SETTINGS", ComponentFocus::Active);
    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Options
            Constraint::Length(1), // Help line
        ])
        .split(inner_area);

    let options = vec![
        ("Retention", format!("{} days", state.config.retention_days)),
        (
            "Compression",
            if state.config.compression_enabled {
                "Enabled".to_string()
            } else {
                "Disabled".to_string()
            },
        ),
        (
            "Mnemosyneignore",
            if state.config.use_mnemosyneignore {
                "Active".to_string()
            } else {
                "Inactive".to_string()
            },
        ),
        ("Theme", THEMES[state.config.theme_index].name.to_string()),
        ("Primary IDE", state.config.ide.as_str().to_string()),
        ("Maintenance", "Run Garbage Collection".to_string()),
        ("Storage", "Clear All History (Danger)".to_string()),
    ];

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let is_selected = i == state.settings_index;
            let style = if is_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_main)
            };

            let content = Line::from(vec![
                Span::styled(format!(" {:<20} ", label), style),
                Span::styled(
                    format!(" │  {}", value),
                    Style::default().fg(theme.text_dim),
                ),
            ]);

            ListItem::new(content).bg(if is_selected { theme.surface } else { theme.bg })
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[0]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(" Navigation: ", Style::default().fg(theme.text_dim)),
        Span::styled("↑↓ arrows", Style::default().fg(theme.text_main)),
        Span::styled("   Action: ", Style::default().fg(theme.text_dim)),
        Span::styled("Enter", Style::default().fg(theme.text_main)),
        Span::styled("   Back: ", Style::default().fg(theme.text_dim)),
        Span::styled("Esc", Style::default().fg(theme.text_main)),
    ]))
    .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(help, chunks[1]);
}
