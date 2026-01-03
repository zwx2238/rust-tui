use crate::render::RenderTheme;
use crate::types::ROLE_USER;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

pub struct MessageLayout {
    pub index: usize,
    pub label_line: usize,
    pub button_range: Option<(usize, usize)>,
}

const EDIT_BUTTON_TEXT: &str = "[编辑]";

pub fn label_line_with_button(role: &str, label: &str, theme: &RenderTheme) -> Line<'static> {
    let label_style = Style::default()
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White))
        .add_modifier(Modifier::BOLD);
    let mut spans = vec![Span::styled(label.to_string(), label_style)];
    if role == ROLE_USER {
        let button_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        spans.push(Span::raw(" "));
        spans.push(Span::styled(EDIT_BUTTON_TEXT.to_string(), button_style));
    }
    Line::from(spans)
}

pub fn label_line_layout(
    role: &str,
    label: &str,
    line_cursor: usize,
) -> (Option<(usize, usize)>, usize) {
    if role != ROLE_USER {
        return (None, line_cursor);
    }
    let label_width = label.width();
    let button_width = EDIT_BUTTON_TEXT.width();
    let start = label_width.saturating_add(1);
    let end = start.saturating_add(button_width);
    (Some((start, end)), line_cursor)
}

#[cfg(test)]
mod tests {
    use super::{label_line_layout, label_line_with_button};
    use crate::render::RenderTheme;
    use ratatui::style::Color;

    #[test]
    fn layout_only_for_user() {
        let (range, cursor) = label_line_layout("assistant", "🤖", 0);
        assert!(range.is_none());
        assert_eq!(cursor, 0);
        let (range, _) = label_line_layout("user", "👤", 0);
        assert!(range.is_some());
    }

    #[test]
    fn label_line_includes_button_for_user() {
        let theme = RenderTheme {
            bg: Color::Black,
            fg: None,
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: None,
        };
        let line = label_line_with_button("user", "👤", &theme);
        let text: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("[编辑]"));
    }
}
