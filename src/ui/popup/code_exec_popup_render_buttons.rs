use crate::render::RenderTheme;
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::state::CodeExecReasonTarget;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders};
use tui_textarea::TextArea;

pub(crate) fn draw_reason_input(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    input: &mut TextArea<'_>,
    target: CodeExecReasonTarget,
    theme: &RenderTheme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let title = match target {
        CodeExecReasonTarget::Deny => "取消原因(可选)",
        CodeExecReasonTarget::Stop => "中止原因(可选)",
    };
    let style = base_style(theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(title))
        .style(style);
    input.set_block(block);
    input.set_style(style);
    input.set_selection_style(Style::default().bg(selection_bg(theme.bg)));
    input.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    input.set_placeholder_text("可填写原因，留空使用默认提示");
    input.set_placeholder_style(Style::default().fg(base_fg(theme)));
    f.render_widget(&*input, area);
}

pub(crate) fn build_title(live: Option<&crate::ui::state::CodeExecLive>) -> String {
    match live {
        Some(live) => build_live_title(live),
        None => "代码执行确认 · 等待确认".to_string(),
    }
}

fn build_live_title(live: &crate::ui::state::CodeExecLive) -> String {
    if live.done || live.exit_code.is_some() {
        let finished_at = live.finished_at.unwrap_or_else(std::time::Instant::now);
        let exec = finished_at.duration_since(live.started_at).as_secs_f32();
        let wait = finished_at.elapsed().as_secs_f32();
        format!("代码执行确认 · 已完成 {:.1}s | 等待 {:.1}s", exec, wait)
    } else {
        let elapsed = live.started_at.elapsed().as_secs_f32();
        format!("代码执行确认 · 执行中 {:.1}s", elapsed)
    }
}

// CodeExec 的按钮渲染已迁移到 framework/widget_system/widgets/code_exec/buttons。
