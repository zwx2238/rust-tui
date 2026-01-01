use crate::render::RenderTheme;
use ratatui::style::{Color, Style};

pub(crate) fn base_fg(theme: &RenderTheme) -> Color {
    theme.fg.unwrap_or(Color::White)
}

pub(crate) fn base_style(theme: &RenderTheme) -> Style {
    Style::default().bg(theme.bg).fg(base_fg(theme))
}

pub(crate) fn focus_border_style(theme: &RenderTheme, focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(base_fg(theme))
    }
}

pub(crate) fn selection_bg(bg: Color) -> Color {
    match bg {
        Color::White => Color::Gray,
        _ => Color::DarkGray,
    }
}
