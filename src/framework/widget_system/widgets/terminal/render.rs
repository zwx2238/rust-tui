use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear};
use tui_term::widget::{Cursor, PseudoTerminal};

use crate::framework::widget_system::context::WidgetFrame;

pub(crate) fn render_terminal_popup(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    popup: Rect,
    terminal_area: Rect,
) {
    // 清掉底层聊天内容，避免 overlay 区域字符“透出”。
    frame.frame.render_widget(Clear, popup);
    render_border(frame, popup);
    render_terminal_contents(frame, terminal_area);
}

fn render_border(frame: &mut WidgetFrame<'_, '_, '_, '_>, popup: Rect) {
    let theme = frame.state.theme;
    let title = "Terminal  F7:关闭  滚轮:回看";
    let border = Block::default().borders(Borders::ALL).title(title).style(
        Style::default()
            .bg(theme.bg)
            .fg(theme.fg.unwrap_or(Color::White)),
    );
    frame.frame.render_widget(border, popup);
}

fn render_terminal_contents(frame: &mut WidgetFrame<'_, '_, '_, '_>, terminal_area: Rect) {
    let theme = frame.state.theme;
    let Some(app) = frame.state.active_app_mut() else {
        return;
    };
    let Some(terminal) = app.terminal.as_mut() else {
        return;
    };
    let cursor = Cursor::default().visibility(false);
    let widget = PseudoTerminal::new(terminal.screen_for_render())
        .style(
            Style::default()
                .bg(theme.bg)
                .fg(theme.fg.unwrap_or(Color::White)),
        )
        .cursor(cursor);
    frame.frame.render_widget(widget, terminal_area);
}
