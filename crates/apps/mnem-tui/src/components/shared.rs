use crate::theme::Theme;
use ratatui::{
    style::{Modifier, Style, Stylize},
    widgets::{Block, Borders, Padding},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentFocus {
    Active,
    Inactive,
}

pub struct ZedBlock;

impl ZedBlock {
    pub fn default<'a, T>(theme: &'a Theme, title: T, focus: ComponentFocus) -> Block<'a>
    where
        T: Into<ratatui::text::Line<'a>>,
    {
        let border_color = match focus {
            ComponentFocus::Active => theme.accent,
            _ => theme.border,
        };

        let title_style = match focus {
            ComponentFocus::Active => Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
            _ => Style::default().fg(theme.text_dim),
        };

        Block::default()
            .borders(Borders::ALL)
            .border_set(ratatui::symbols::border::PLAIN)
            .border_style(Style::default().fg(border_color))
            .title(title)
            .title_style(title_style)
            .bg(theme.bg)
    }

    pub fn ghost<'a>(theme: &'a Theme) -> Block<'a> {
        Block::default()
            .bg(theme.bg)
            .padding(Padding::horizontal(1))
    }
}

pub fn get_selection_style(theme: &Theme, is_focused: bool) -> Style {
    if is_focused {
        Style::default()
            .bg(theme.surface)
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_main)
    }
}
