use crate::render::theme::RenderTheme;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use textwrap::wrap;

pub(crate) fn render_list_item_lines(
    text: &str,
    prefix: &str,
    width: usize,
    theme: &RenderTheme,
) -> Vec<Line<'static>> {
    let style = Style::default().fg(theme.fg.unwrap_or(Color::White));
    let available = width.max(10).saturating_sub(prefix.chars().count());
    let wrapped = wrap(text, available.max(1));
    wrapped
        .into_iter()
        .enumerate()
        .map(|(idx, line)| {
            let content = if idx == 0 {
                format!("{prefix}{line}")
            } else {
                format!("{}{}", " ".repeat(prefix.chars().count()), line)
            };
            Line::from(Span::styled(content, style))
        })
        .collect()
}

pub(crate) fn count_list_item_lines(text: &str, prefix: &str, width: usize) -> usize {
    let available = width.max(10).saturating_sub(prefix.chars().count());
    wrap(text, available.max(1)).len()
}
