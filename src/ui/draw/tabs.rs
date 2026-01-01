use crate::render::RenderTheme;
use crate::ui::draw::style::base_fg;
use crate::ui::logic::tab_label;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

pub(crate) fn draw_tabs(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    tabs_len: usize,
    active_tab: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
) {
    let mut label = String::new();
    for i in 0..tabs_len {
        let tab = tab_label(i);
        label.push_str(&tab);
        if i + 1 < tabs_len {
            label.push('│');
        }
    }
    let mut spans = Vec::new();
    let mut cursor = 0usize;
    for (i, part) in label.split('│').enumerate() {
        let style = if i == active_tab {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(base_fg(theme))
        };
        spans.push(Span::styled(part.to_string(), style));
        cursor += part.width();
        if i + 1 < tabs_len {
            spans.push(Span::styled("│", Style::default().fg(base_fg(theme))));
            cursor += 1;
        }
    }
    if let Some(text) = startup_text {
        let width = area.width as usize;
        let text_width = text.width();
        if width > cursor + text_width {
            let pad = width.saturating_sub(cursor + text_width);
            spans.push(Span::raw(" ".repeat(pad)));
            spans.push(Span::styled(
                text.to_string(),
                Style::default().fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White)),
            ));
        }
    }
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg));
    f.render_widget(paragraph, area);
}
