use crate::render::RenderTheme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::block::Padding;
use ratatui::widgets::{Block, Borders};
use tui_textarea::TextArea;

const PADDING_X: u16 = 1;
const PADDING_Y: u16 = 0;

pub(crate) fn draw_input(
    f: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    input: &mut TextArea<'_>,
    theme: &RenderTheme,
    focused: bool,
    busy: bool,
    model_key: &str,
) {
    let style = Style::default()
        .bg(theme.bg)
        .fg(theme.fg.unwrap_or(Color::White));
    let border_style = if focused {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(theme.fg.unwrap_or(Color::White))
    };
    let (line_idx, col) = input.cursor();
    let total_lines = input.lines().len().max(1);
    let status = format!(
        "{} · 模型 {} · 行 {}/{} 列 {}",
        if busy { "输入(禁用)" } else { "输入" },
        model_key,
        line_idx + 1,
        total_lines,
        col + 1
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(status)
        .title_top(Line::from("Enter 发送 · Ctrl+J 换行").right_aligned())
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style);
    input.set_block(block);
    input.set_style(style);
    input.set_selection_style(Style::default().bg(Color::DarkGray));
    input.set_cursor_style(if focused && !busy {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    });
    input.set_placeholder_text(if busy {
        "正在生成回复，输入已禁用"
    } else {
        "输入内容后按 Enter 发送"
    });
    input.set_placeholder_style(Style::default().fg(Color::DarkGray));
    f.render_widget(&*input, area);
}
