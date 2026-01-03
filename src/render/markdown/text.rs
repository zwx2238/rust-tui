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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::theme::RenderTheme;
    use ratatui::style::Color;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    #[test]
    fn renders_paragraph_wrapped() {
        let lines = render_paragraph_lines("hello world from deepchat", 8, &theme());
        assert!(lines.len() >= 2);
    }

    #[test]
    fn renders_heading_with_style() {
        let lines = render_heading_lines("Heading", HeadingLevel::H2, 10, &theme());
        assert_eq!(lines.len(), 1);
        assert!(lines[0].to_string().contains("Heading"));
    }
}
