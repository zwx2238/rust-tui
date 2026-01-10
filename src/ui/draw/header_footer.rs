use crate::render::RenderTheme;
use crate::ui::draw::style::base_fg;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(crate) fn draw_header(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    theme: &RenderTheme,
    note: Option<&str>,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White));
    let text = if let Some(note) = note {
        format!("deepchat  ·  {note}")
    } else {
        "deepchat".to_string()
    };
    let line = Line::from(Span::styled(text, style));
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

pub(crate) fn draw_footer(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    theme: &RenderTheme,
    nav_mode: bool,
    follow: bool,
) {
    let time = chrono::Local::now().format("%H:%M:%S").to_string();
    let mut parts = vec![time];
    if nav_mode {
        parts.push("NAV".to_string());
    }
    let follow_text = if follow { "追底:开" } else { "追底:关" };
    parts.push(follow_text.to_string());
    let text = parts.join("  ");
    let style = Style::default().bg(theme.bg).fg(base_fg(theme));
    let line = Line::from(Span::styled(text, style));
    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}
