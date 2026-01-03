use crate::render::theme::RenderTheme;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use textwrap::wrap;
use unicode_width::UnicodeWidthStr;

pub(crate) fn render_list_item_lines(
    text: &str,
    prefix: &str,
    indent: &str,
    width: usize,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
    let style = Style::default().fg(theme.fg.unwrap_or(Color::White));
    let indent_width = UnicodeWidthStr::width(indent);
    let prefix_width = UnicodeWidthStr::width(prefix);
    let available = width.max(10).saturating_sub(indent_width + prefix_width);
    let wrapped = wrap(text, available.max(1));
    wrapped
        .into_iter()
        .enumerate()
        .map(|(idx, line)| {
            let content = if idx == 0 {
                format!("{indent}{prefix}{line}")
            } else {
                format!("{indent}{}{line}", " ".repeat(prefix_width))
            };
            Line::from(Span::styled(content, style))
        })
        .collect()
}

pub(crate) fn count_list_item_lines(text: &str, prefix: &str, indent: &str, width: usize) -> usize {
    let indent_width = UnicodeWidthStr::width(indent);
    let prefix_width = UnicodeWidthStr::width(prefix);
    let available = width.max(10).saturating_sub(indent_width + prefix_width);
    wrap(text, available.max(1)).len()
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
    fn renders_wrapped_list_items() {
        let lines = render_list_item_lines(
            "这是一个很长的列表项，需要换行",
            "1. ",
            "  ",
            10,
            &theme(),
        );
        assert!(lines.len() >= 2);
        assert!(lines[0].to_string().contains("1."));
    }

    #[test]
    fn counts_list_item_lines() {
        let count = count_list_item_lines("long long long long", "- ", "", 8);
        assert!(count >= 2);
    }
}
