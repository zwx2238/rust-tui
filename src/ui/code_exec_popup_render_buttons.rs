use crate::render::RenderTheme;
use crate::ui::code_exec_popup_layout::CodeExecPopupLayout;
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::state::{CodeExecHover, CodeExecReasonTarget};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use tui_textarea::TextArea;

pub(crate) fn render_action_buttons(
    f: &mut ratatui::Frame<'_>,
    layout: CodeExecPopupLayout,
    hover: Option<CodeExecHover>,
    reason_target: Option<CodeExecReasonTarget>,
    live: Option<&crate::ui::state::CodeExecLive>,
    theme: &RenderTheme,
) {
    let finished = live
        .map(|l| l.done || l.exit_code.is_some())
        .unwrap_or(false);
    let running = live.is_some() && !finished;
    if let Some(target) = reason_target {
        render_reason_buttons(f, layout, target, hover, theme);
        return;
    }
    if finished {
        render_exit_button(f, layout, hover, theme);
        return;
    }
    if running {
        render_stop_button(f, layout, hover, theme);
        return;
    }
    render_approve_deny_buttons(f, layout, hover, theme);
}

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

fn render_reason_buttons(
    f: &mut ratatui::Frame<'_>,
    layout: CodeExecPopupLayout,
    target: CodeExecReasonTarget,
    hover: Option<CodeExecHover>,
    theme: &RenderTheme,
) {
    let confirm_label = match target {
        CodeExecReasonTarget::Deny => "确认取消",
        CodeExecReasonTarget::Stop => "确认中止",
    };
    render_button(
        f,
        layout.approve_btn,
        confirm_label,
        button_style(hover, CodeExecHover::ReasonConfirm, theme),
    );
    render_button(
        f,
        layout.deny_btn,
        "返回",
        button_style(hover, CodeExecHover::ReasonBack, theme),
    );
}

fn render_exit_button(
    f: &mut ratatui::Frame<'_>,
    layout: CodeExecPopupLayout,
    hover: Option<CodeExecHover>,
    theme: &RenderTheme,
) {
    render_button(
        f,
        layout.exit_btn,
        "退出",
        button_style(hover, CodeExecHover::Exit, theme),
    );
}

fn render_stop_button(
    f: &mut ratatui::Frame<'_>,
    layout: CodeExecPopupLayout,
    hover: Option<CodeExecHover>,
    theme: &RenderTheme,
) {
    render_button(
        f,
        layout.stop_btn,
        "停止执行",
        button_style(hover, CodeExecHover::Stop, theme),
    );
}

fn render_approve_deny_buttons(
    f: &mut ratatui::Frame<'_>,
    layout: CodeExecPopupLayout,
    hover: Option<CodeExecHover>,
    theme: &RenderTheme,
) {
    render_button(
        f,
        layout.approve_btn,
        "确认执行",
        button_style(hover, CodeExecHover::Approve, theme),
    );
    render_button(
        f,
        layout.deny_btn,
        "取消拒绝",
        button_style(hover, CodeExecHover::Deny, theme),
    );
}

fn render_button(f: &mut ratatui::Frame<'_>, area: Rect, label: &str, style: Style) {
    let block = Block::default().borders(Borders::ALL).style(style);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(Line::from(label))
            .style(style)
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

fn button_style(hover: Option<CodeExecHover>, target: CodeExecHover, theme: &RenderTheme) -> Style {
    match hover {
        Some(h) if h == target => Style::default()
            .bg(selection_bg(theme.bg))
            .fg(base_fg(theme))
            .add_modifier(Modifier::BOLD),
        _ => base_style(theme),
    }
}
