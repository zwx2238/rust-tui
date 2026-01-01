use crate::render::RenderTheme;
use crate::ui::draw::style::base_fg;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(crate) fn draw_header(f: &mut ratatui::Frame<'_>, area: Rect, theme: &RenderTheme) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.heading_fg.or(theme.fg).unwrap_or(Color::White));
    let line = Line::from(Span::styled("deepchat", style));
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

pub(crate) fn draw_footer(f: &mut ratatui::Frame<'_>, area: Rect, theme: &RenderTheme) {
    let time = chrono::Local::now().format("%H:%M:%S").to_string();
    let style = Style::default()
        .bg(theme.bg)
        .fg(base_fg(theme));
    let line = Line::from(Span::styled(time, style));
    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}
