use crate::ui::code_exec_popup_layout::{CodeExecPopupLayout, code_exec_popup_layout};
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use crate::ui::selection::{Selection, apply_selection_to_text};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::state::{CodeExecHover, CodeExecReasonTarget, PendingCodeExec};
use crate::framework::widget_system::context::{UpdateOutput, WidgetFrame};
use std::error::Error;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use tui_textarea::TextArea;

use super::buttons::render_buttons;
use super::helpers::{point_in_rect, snapshot_live};
use super::widget::CodeExecWidget;

pub(super) fn render_code_exec_overlay(
    widget: &mut CodeExecWidget,
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: ratatui::layout::Rect,
) -> Result<(), Box<dyn Error>> {
    let active_tab = frame.state.active_tab;
    let pending = match frame
        .state
        .tabs
        .get(active_tab)
        .and_then(|tab_state| tab_state.app.pending_code_exec.clone())
    {
        Some(pending) => pending,
        None => return Ok(()),
    };
    let tab_state = frame
        .state
        .tabs
        .get_mut(active_tab)
        .expect("active_tab should remain valid");
    let popup = code_exec_popup_layout(rect, tab_state.app.code_exec_reason_target.is_some());
    let live_snapshot = prepare_code_exec_overlay(frame.state.theme, tab_state, &pending, popup);
    let hover = tab_state.app.code_exec_hover;
    let reason_target = tab_state.app.code_exec_reason_target;
    let mut reason_input = std::mem::take(&mut tab_state.app.code_exec_reason_input);
    {
    let mut params = build_params(
        rect,
        frame.state.theme,
        tab_state,
        &pending,
        live_snapshot.as_ref(),
            &mut reason_input,
        );
        draw_code_exec_popup_base(frame.frame, &mut params);
    }
    tab_state.app.code_exec_reason_input = reason_input;
    render_buttons(
        widget,
        frame,
        super::buttons::CodeExecButtonsRenderParams {
            area: rect,
            hover,
            reason_target,
            live: live_snapshot.as_ref(),
            theme: frame.state.theme,
            layout,
            update,
        },
    );
    Ok(())
}

pub(super) fn hover_at(
    m: crossterm::event::MouseEvent,
    popup: CodeExecPopupLayout,
    reason_mode: bool,
) -> Option<CodeExecHover> {
    if point_in_rect(m.column, m.row, popup.approve_btn) {
        return Some(if reason_mode {
            CodeExecHover::ReasonConfirm
        } else {
            CodeExecHover::Approve
        });
    }
    if point_in_rect(m.column, m.row, popup.deny_btn) {
        return Some(if reason_mode {
            CodeExecHover::ReasonBack
        } else {
            CodeExecHover::Deny
        });
    }
    if point_in_rect(m.column, m.row, popup.stop_btn) {
        return Some(CodeExecHover::Stop);
    }
    if point_in_rect(m.column, m.row, popup.exit_btn) {
        return Some(CodeExecHover::Exit);
    }
    None
}

fn build_params<'a>(
    area: ratatui::layout::Rect,
    theme: &'a crate::render::RenderTheme,
    tab_state: &'a mut crate::ui::runtime_helpers::TabState,
    pending: &'a PendingCodeExec,
    live: Option<&'a crate::ui::state::CodeExecLive>,
    reason_input: &'a mut tui_textarea::TextArea<'static>,
) -> CodeExecPopupParams<'a, 'static> {
    CodeExecPopupParams {
        area,
        pending,
        scroll: tab_state.app.code_exec_scroll,
        stdout_scroll: tab_state.app.code_exec_stdout_scroll,
        stderr_scroll: tab_state.app.code_exec_stderr_scroll,
        reason_target: tab_state.app.code_exec_reason_target,
        reason_input,
        live,
        code_selection: tab_state.app.code_exec_code_selection,
        stdout_selection: tab_state.app.code_exec_stdout_selection,
        stderr_selection: tab_state.app.code_exec_stderr_selection,
        theme,
    }
}

fn prepare_code_exec_overlay(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    layout: CodeExecPopupLayout,
) -> Option<crate::ui::state::CodeExecLive> {
    let (stdout, stderr, live_snapshot) = snapshot_live(tab_state);
    clamp_code_scroll(theme, tab_state, pending, layout);
    clamp_output_scrolls(theme, tab_state, &stdout, &stderr, layout);
    live_snapshot
}

fn clamp_code_scroll(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    layout: CodeExecPopupLayout,
) {
    let max_scroll = code_max_scroll(
        &pending.code,
        layout.code_text_area.width,
        layout.code_text_area.height,
        theme,
    );
    if tab_state.app.code_exec_scroll > max_scroll {
        tab_state.app.code_exec_scroll = max_scroll;
    }
}

fn clamp_output_scrolls(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    stdout: &str,
    stderr: &str,
    layout: CodeExecPopupLayout,
) {
    let max_stdout = stdout_max_scroll(
        stdout,
        layout.stdout_text_area.width,
        layout.stdout_text_area.height,
        theme,
    );
    let max_stderr = stderr_max_scroll(
        stderr,
        layout.stderr_text_area.width,
        layout.stderr_text_area.height,
        theme,
    );
    if tab_state.app.code_exec_stdout_scroll > max_stdout {
        tab_state.app.code_exec_stdout_scroll = max_stdout;
    }
    if tab_state.app.code_exec_stderr_scroll > max_stderr {
        tab_state.app.code_exec_stderr_scroll = max_stderr;
    }
}

struct CodeExecPopupParams<'a, 'b> {
    area: Rect,
    pending: &'a PendingCodeExec,
    scroll: usize,
    stdout_scroll: usize,
    stderr_scroll: usize,
    reason_target: Option<CodeExecReasonTarget>,
    reason_input: &'a mut TextArea<'b>,
    live: Option<&'a crate::ui::state::CodeExecLive>,
    code_selection: Option<Selection>,
    stdout_selection: Option<Selection>,
    stderr_selection: Option<Selection>,
    theme: &'a crate::render::RenderTheme,
}

fn draw_code_exec_popup_base<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &mut CodeExecPopupParams<'a, 'b>,
) {
    let layout = code_exec_popup_layout(params.area, params.reason_target.is_some());
    render_popup_base_layer(f, params, layout);
    render_panels(f, params, layout);
    render_reason_if_needed(f, params, layout);
}

fn render_popup_base_layer<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &CodeExecPopupParams<'a, 'b>,
    layout: CodeExecPopupLayout,
) {
    let mask = popup_mask(params.area, layout.popup);
    render_mask(f, params.theme, mask);
    render_popup_base(f, params.theme, layout.popup, &build_title(params.live));
}

fn render_panels<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &CodeExecPopupParams<'a, 'b>,
    layout: CodeExecPopupLayout,
) {
    render_code_panel(
        f,
        params.theme,
        params.pending,
        layout,
        params.scroll,
        params.code_selection,
    );
    render_stdout_panel(
        f,
        params.theme,
        layout,
        params.live,
        params.stdout_scroll,
        params.stdout_selection,
    );
    render_stderr_panel(
        f,
        params.theme,
        layout,
        params.live,
        params.stderr_scroll,
        params.stderr_selection,
    );
}

fn render_reason_if_needed<'a, 'b>(
    f: &mut ratatui::Frame<'_>,
    params: &mut CodeExecPopupParams<'a, 'b>,
    layout: CodeExecPopupLayout,
) {
    if let Some(target) = params.reason_target {
        draw_reason_input(
            f,
            layout.reason_input_area,
            params.reason_input,
            target,
            params.theme,
        );
    }
}

fn popup_mask(area: Rect, popup: Rect) -> Rect {
    let max_x = area.x.saturating_add(area.width);
    let max_y = area.y.saturating_add(area.height);
    let mask_x = popup
        .x
        .saturating_sub(crate::ui::code_exec_popup_layout::OUTER_MARGIN)
        .max(area.x);
    let mask_y = popup
        .y
        .saturating_sub(crate::ui::code_exec_popup_layout::OUTER_MARGIN)
        .max(area.y);
    let mask_w = popup
        .width
        .saturating_add(crate::ui::code_exec_popup_layout::OUTER_MARGIN.saturating_mul(2))
        .min(max_x.saturating_sub(mask_x));
    let mask_h = popup
        .height
        .saturating_add(crate::ui::code_exec_popup_layout::OUTER_MARGIN.saturating_mul(2))
        .min(max_y.saturating_sub(mask_y));
    Rect {
        x: mask_x,
        y: mask_y,
        width: mask_w,
        height: mask_h,
    }
}

fn render_mask(f: &mut ratatui::Frame<'_>, theme: &crate::render::RenderTheme, mask: Rect) {
    f.render_widget(Clear, mask);
    let mask_block = Block::default().style(base_style(theme));
    f.render_widget(mask_block, mask);
}

fn render_popup_base(
    f: &mut ratatui::Frame<'_>,
    theme: &crate::render::RenderTheme,
    popup: Rect,
    title: &str,
) {
    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(base_fg(theme))
                .add_modifier(Modifier::BOLD),
        )]))
        .style(base_style(theme))
        .border_style(Style::default().fg(base_fg(theme)));
    f.render_widget(block, popup);
}

fn render_code_panel(
    f: &mut ratatui::Frame<'_>,
    theme: &crate::render::RenderTheme,
    pending: &PendingCodeExec,
    layout: CodeExecPopupLayout,
    scroll: usize,
    selection: Option<Selection>,
) {
    let (text, total_lines) = crate::ui::code_exec_popup_text::build_code_text(
        &pending.code,
        layout.code_text_area.width,
        layout.code_text_area.height,
        scroll,
        theme,
    );
    let text = apply_selection_if_needed(text, scroll, selection);
    render_text_panel(
        f,
        TextPanelParams {
            theme,
            text,
            area: layout.code_text_area,
            scrollbar_area: layout.code_scrollbar_area,
            total_lines,
            scroll,
            title: None,
        },
    );
}

fn render_stdout_panel(
    f: &mut ratatui::Frame<'_>,
    theme: &crate::render::RenderTheme,
    layout: CodeExecPopupLayout,
    live: Option<&crate::ui::state::CodeExecLive>,
    scroll: usize,
    selection: Option<Selection>,
) {
    let (text, total_lines) = crate::ui::code_exec_popup_text::build_stdout_text(
        live.map(|l| l.stdout.as_str()),
        layout.stdout_text_area.width,
        layout.stdout_text_area.height,
        scroll,
        theme,
    );
    let text = apply_selection_if_needed(text, scroll, selection);
    render_text_panel(
        f,
        TextPanelParams {
            theme,
            text,
            area: layout.stdout_text_area,
            scrollbar_area: layout.stdout_scrollbar_area,
            total_lines,
            scroll,
            title: Some("STDOUT"),
        },
    );
}

fn render_stderr_panel(
    f: &mut ratatui::Frame<'_>,
    theme: &crate::render::RenderTheme,
    layout: CodeExecPopupLayout,
    live: Option<&crate::ui::state::CodeExecLive>,
    scroll: usize,
    selection: Option<Selection>,
) {
    let (text, total_lines) = crate::ui::code_exec_popup_text::build_stderr_text(
        live.map(|l| l.stderr.as_str()),
        layout.stderr_text_area.width,
        layout.stderr_text_area.height,
        scroll,
        theme,
    );
    let text = apply_selection_if_needed(text, scroll, selection);
    render_text_panel(
        f,
        TextPanelParams {
            theme,
            text,
            area: layout.stderr_text_area,
            scrollbar_area: layout.stderr_scrollbar_area,
            total_lines,
            scroll,
            title: Some("STDERR"),
        },
    );
}

struct TextPanelParams<'a> {
    theme: &'a crate::render::RenderTheme,
    text: Text<'a>,
    area: Rect,
    scrollbar_area: Rect,
    total_lines: usize,
    scroll: usize,
    title: Option<&'a str>,
}

fn render_text_panel(f: &mut ratatui::Frame<'_>, params: TextPanelParams<'_>) {
    let block = match params.title {
        Some(title) => Block::default().borders(Borders::NONE).title_top(title),
        None => Block::default().borders(Borders::NONE),
    };
    let para = Paragraph::new(params.text)
        .style(base_style(params.theme))
        .block(block);
    f.render_widget(para, params.area);
    render_scrollbar_if_needed(
        f,
        params.theme,
        params.area,
        params.scrollbar_area,
        params.total_lines,
        params.scroll,
    );
}

fn apply_selection_if_needed(
    text: Text<'static>,
    scroll: usize,
    selection: Option<Selection>,
) -> Text<'static> {
    let Some(selection) = selection else {
        return text;
    };
    if selection.is_empty() {
        return text;
    }
    apply_selection_to_text(
        &text,
        scroll,
        selection,
        Style::default().bg(Color::DarkGray),
    )
}

fn render_scrollbar_if_needed(
    f: &mut ratatui::Frame<'_>,
    theme: &crate::render::RenderTheme,
    area: Rect,
    scrollbar_area: Rect,
    total_lines: usize,
    scroll: usize,
) {
    if total_lines <= area.height as usize {
        return;
    }
    let viewport_len = area.height as usize;
    let max_scroll = total_lines.saturating_sub(viewport_len);
    let mut state = ScrollbarState::new(max_scroll.saturating_add(1))
        .position(scroll.min(max_scroll))
        .viewport_content_length(viewport_len);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(base_fg(theme)))
        .track_style(Style::default().fg(base_fg(theme)));
    f.render_stateful_widget(scrollbar, scrollbar_area, &mut state);
}

fn draw_reason_input(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    input: &mut TextArea<'_>,
    target: CodeExecReasonTarget,
    theme: &crate::render::RenderTheme,
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

fn build_title(live: Option<&crate::ui::state::CodeExecLive>) -> String {
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
