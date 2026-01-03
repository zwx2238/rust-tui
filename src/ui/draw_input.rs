use crate::render::RenderTheme;
use crate::ui::draw::layout::{PADDING_X, PADDING_Y};
use crate::ui::draw::style::{base_style, focus_border_style};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::block::Padding;
use ratatui::widgets::{Block, Borders};
use tui_textarea::TextArea;

pub(crate) struct InputDrawParams<'a, 'b> {
    pub area: ratatui::layout::Rect,
    pub input: &'a mut TextArea<'b>,
    pub theme: &'a RenderTheme,
    pub focused: bool,
    pub busy: bool,
    pub model_key: &'a str,
    pub prompt_key: &'a str,
}

pub(crate) fn draw_input<'a, 'b>(f: &mut ratatui::Frame<'_>, params: InputDrawParams<'a, 'b>) {
    let style = base_style(params.theme);
    let border_style = focus_border_style(params.theme, params.focused);
    let status = build_status(
        params.input,
        params.busy,
        params.model_key,
        params.prompt_key,
    );
    let block = build_block(status, style, border_style);
    params.input.set_block(block);
    params.input.set_style(style);
    params
        .input
        .set_selection_style(Style::default().bg(Color::DarkGray));
    params
        .input
        .set_cursor_style(cursor_style(params.focused, params.busy));
    params
        .input
        .set_placeholder_text(placeholder_text(params.busy));
    params
        .input
        .set_placeholder_style(Style::default().fg(Color::DarkGray));
    f.render_widget(&*params.input, params.area);
}

fn build_status(input: &TextArea<'_>, busy: bool, model_key: &str, prompt_key: &str) -> String {
    let (line_idx, col) = input.cursor();
    let total_lines = input.lines().len().max(1);
    format!(
        "{} · 模型 {} · 角色 {} · 行 {}/{} 列 {}",
        if busy { "输入(禁用)" } else { "输入" },
        model_key,
        prompt_key,
        line_idx + 1,
        total_lines,
        col + 1
    )
}

fn build_block(status: String, style: Style, border_style: Style) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title_top(status)
        .title_top(Line::from("Enter 发送 · Ctrl+J 换行").right_aligned())
        .padding(Padding::new(PADDING_X, PADDING_X, PADDING_Y, PADDING_Y))
        .style(style)
        .border_style(border_style)
}

fn cursor_style(focused: bool, busy: bool) -> Style {
    if focused && !busy {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    }
}

fn placeholder_text(busy: bool) -> &'static str {
    if busy {
        "正在生成回复，输入已禁用"
    } else {
        "输入内容后按 Enter 发送"
    }
}
