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
