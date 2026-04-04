use crate::app::{AppState, Focus, HistoryItem};
use crate::components::shared::{ComponentFocus, ZedBlock};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
};

fn get_file_icon(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "",
        "toml" | "yaml" | "yml" | "json" => "",
        "ts" | "tsx" | "js" | "jsx" => "",
        "py" => "",
        "go" => "",
        "c" | "h" | "cpp" | "cc" | "cxx" => "",
        "java" | "kt" | "kts" => "",
        "md" | "mdx" => "",
        "html" | "htm" => "",
        "css" | "scss" | "sass" => "",
        "sh" | "bash" | "zsh" => "",
        "lock" => "",
        _ => "",
    }
}

pub fn render_files(f: &mut Frame, area: Rect, state: &mut AppState) {
    let is_focused = state.focus == Focus::Files;
    let theme = &state.theme;

    let block = ZedBlock::default(
        theme,
        " FILES ",
        if is_focused {
            ComponentFocus::Active
        } else {
            ComponentFocus::Inactive
        },
    );

    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let name = std::path::Path::new(&file.path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            let is_selected = state.files_state.selected() == Some(idx);

            let icon = get_file_icon(&file.path);

            let mut name_style = if is_selected && is_focused {
                Style::default()
                    .fg(theme.bg)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text_main)
            };

            if !is_focused && !is_selected {
                name_style = name_style.fg(theme.text_dim);
            }

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", icon),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(name.to_string(), name_style),
            ]))
        })
        .collect();

    let list = List::new(items).block(block).highlight_symbol("");

    f.render_stateful_widget(list, area, &mut state.files_state);
}

pub fn render_timeline(f: &mut Frame, area: Rect, state: &mut AppState) {
    let is_focused = state.focus == Focus::Timeline;
    let theme = &state.theme;

    let block = ZedBlock::default(
        theme,
        " SNAPSHOTS ",
        if is_focused {
            ComponentFocus::Active
        } else {
            ComponentFocus::Inactive
        },
    );

    let items: Vec<ListItem> = state
        .history_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = state.versions_state.selected() == Some(i);

            match item {
                HistoryItem::DateHeader(date) => ListItem::new(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        format!("── {} ──", date),
                        Style::default().fg(theme.text_dim),
                    ),
                ])),
                HistoryItem::Session(session) => {
                    let mut style = Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD);
                    if is_selected && is_focused {
                        style = style.fg(theme.bg).bg(theme.accent);
                    }
                    ListItem::new(Line::from(vec![
                        Span::styled("┊ ", Style::default().fg(theme.text_dim)),
                        Span::styled("╭┄ ", Style::default().fg(theme.text_dim)),
                        Span::styled(format!("{} ", session.label), style),
                    ]))
                }
                HistoryItem::Snapshot(s) => {
                    let time = chrono::DateTime::parse_from_rfc3339(&s.timestamp)
                        .map(|dt| dt.format("%H:%M").to_string())
                        .unwrap_or_else(|_| s.timestamp.clone());

                    let hash_short = if s.content_hash.len() > 7 {
                        &s.content_hash[..7]
                    } else {
                        &s.content_hash
                    };

                    let style = if is_selected && is_focused {
                        Style::default().fg(theme.bg).bg(theme.accent)
                    } else if is_selected {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text_dim)
                    };

                    let is_base = state.diff_base_hash.as_ref() == Some(&s.content_hash);
                    let is_last_in_session = state.history_items.get(i + 1).is_none_or(|next| {
                        matches!(next, HistoryItem::Session(_) | HistoryItem::DateHeader(_))
                    });

                    let mut spans = vec![
                        Span::styled("┊ ", Style::default().fg(theme.text_dim)),
                        Span::styled("● ", Style::default().fg(theme.accent)),
                        Span::styled(
                            format!("[{}] ", hash_short),
                            Style::default().fg(theme.accent),
                        ),
                        Span::styled(format!("{} ", time), style),
                    ];

                    if is_base {
                        spans.push(Span::styled(
                            "COMPARE ",
                            Style::default()
                                .fg(theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }

                    spans.push(Span::styled(
                        s.git_branch.clone().unwrap_or_else(|| "".into()),
                        Style::default()
                            .fg(theme.text_dim)
                            .add_modifier(Modifier::ITALIC),
                    ));

                    if is_last_in_session {
                        // We could add the closing connector here or in a separate item.
                        // For simplicity in a single ListItem:
                        ListItem::new(Line::from(spans))
                    } else {
                        ListItem::new(Line::from(spans))
                    }
                }
            }
        })
        .collect();

    let list = List::new(items).block(block).highlight_symbol("");

    f.render_stateful_widget(list, area, &mut state.versions_state);
}
