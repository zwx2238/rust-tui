use crate::render::theme::RenderTheme;
use pulldown_cmark::HeadingLevel;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use textwrap::wrap;

pub(crate) fn render_paragraph_lines(text: &str, width: usize, theme: &RenderTheme) -> Vec<Line<'static>> {
    let style = Style::default().fg(theme.fg.unwrap_or(Color::White));
    wrap(text, width.max(10))
        .into_iter()
        .map(|line| Line::from(Span::styled(line.to_string(), style)))
        .collect()
}

pub(crate) fn render_heading_lines(
    text: &str,
    level: HeadingLevel,
    width: usize,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
    let ch = match level {
        HeadingLevel::H1 => '=',
        HeadingLevel::H2 => '-',
        HeadingLevel::H3 => '~',
        _ => '.',
    };
    let rule = ch.to_string().repeat(width.max(10));
    let style = Style::default()
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White))
        .add_modifier(Modifier::BOLD);
    vec![
        Line::from(Span::styled(rule.clone(), style)),
        Line::from(Span::styled(text.to_string(), style)),
        Line::from(Span::styled(rule, style)),
    ]
}
