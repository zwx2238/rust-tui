use crate::render::RenderTheme;
use crate::ui::draw::style::base_fg;
use crate::ui::text_utils::truncate_to_width;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(crate) fn draw_categories(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    categories: &[String],
    active: usize,
    theme: &RenderTheme,
) {
    let mut lines = Vec::new();
    let width = area.width.saturating_sub(2).max(1) as usize;
    for (idx, name) in categories.iter().enumerate() {
        let prefix = if idx == active { "● " } else { "  " };
        let label = truncate_to_width(name, width.saturating_sub(2));
        let text = format!("{prefix}{label}");
        let style = if idx == active {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(base_fg(theme))
        };
        lines.push(Line::from(Span::styled(text, style)));
    }
    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(theme.bg));
    f.render_widget(paragraph, area);
}
