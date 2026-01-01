use crate::render::theme::RenderTheme;
use pulldown_cmark::HeadingLevel;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use textwrap::wrap;

pub(crate) fn render_paragraph_lines(
    text: &str,
    width: usize,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
    let style = Style::default().fg(theme.fg.unwrap_or(Color::White));
    wrap(text, width.max(10))
        .into_iter()
        .map(|line| Line::from(Span::styled(line.to_string(), style)))
        .collect()
}

pub(crate) fn render_heading_lines(
    text: &str,
    _level: HeadingLevel,
    width: usize,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
    let style = Style::default()
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White))
        .add_modifier(Modifier::BOLD);
    wrap(text, width.max(10))
        .into_iter()
        .map(|line| Line::from(Span::styled(line.to_string(), style)))
        .collect()
}
