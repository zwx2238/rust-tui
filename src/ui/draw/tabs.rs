use crate::render::RenderTheme;
use crate::ui::draw::style::base_fg;
use crate::ui::tab_bar::build_tab_bar_view;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

pub(crate) fn draw_tabs(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    labels: &[String],
    active_tab: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
) {
    let view = build_tab_bar_view(labels, active_tab, area.width);
    let mut spans = build_tab_spans(&view, theme);
    append_startup_text(&mut spans, area.width as usize, startup_text, theme);
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg));
    f.render_widget(paragraph, area);
}

fn build_tab_spans(
    view: &crate::ui::tab_bar::TabBarView,
    theme: &RenderTheme,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, item) in view.items.iter().enumerate() {
        let style = if item.active {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(base_fg(theme))
        };
        spans.push(Span::styled(item.label.clone(), style));
        if i + 1 < view.items.len() {
            spans.push(Span::styled("│", Style::default().fg(base_fg(theme))));
        }
    }
    spans
}

fn append_startup_text(
    spans: &mut Vec<Span<'static>>,
    width: usize,
    startup_text: Option<&str>,
    theme: &RenderTheme,
) {
    let Some(text) = startup_text else {
        return;
    };
    let cursor = spans.iter().map(|s| s.content.width()).sum::<usize>();
    let text_width = text.width();
    if width <= cursor + text_width {
        return;
    }
    let pad = width.saturating_sub(cursor + text_width);
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(
        text.to_string(),
        Style::default().fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White)),
    ));
}
