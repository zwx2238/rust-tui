use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::widget_system::context::WidgetFrame;

pub(crate) fn render_terminal_popup(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    popup: Rect,
    terminal_area: Rect,
) {
    render_border(frame, popup);
    render_terminal_contents(frame, terminal_area);
}

fn render_border(frame: &mut WidgetFrame<'_, '_, '_, '_>, popup: Rect) {
    let theme = frame.state.theme;
    let title = "Terminal  F7:关闭  滚轮:回看";
    let border = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(theme.bg).fg(theme.fg.unwrap_or(Color::White)));
    frame.frame.render_widget(border, popup);
}

fn render_terminal_contents(frame: &mut WidgetFrame<'_, '_, '_, '_>, terminal_area: Rect) {
    let theme = frame.state.theme;
    let Some(app) = frame.state.active_app() else {
        return;
    };
    let Some(terminal) = app.terminal.as_ref() else {
        return;
    };
    let text = screen_to_text(terminal.screen(), terminal_area.height as usize, terminal.scroll_offset);
    let para = Paragraph::new(text)
        .style(Style::default().bg(theme.bg).fg(theme.fg.unwrap_or(Color::White)))
        .wrap(Wrap { trim: false });
    frame.frame.render_widget(para, terminal_area);
}

fn screen_to_text(screen: &vt100::Screen, height: usize, scroll_offset: u16) -> Text<'static> {
    let all = split_lines(screen.contents());
    let end = all.len().saturating_sub(scroll_offset as usize);
    let start = end.saturating_sub(height);
    let mut lines = Vec::new();
    for s in &all[start..end] {
        lines.push(Line::from((*s).to_string()));
    }
    Text::from(lines)
}

fn split_lines(s: String) -> Vec<String> {
    let mut out = Vec::new();
    for line in s.split('\n') {
        out.push(line.to_string());
    }
    out
}

