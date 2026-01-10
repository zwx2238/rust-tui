use crate::ui::code_exec_popup::draw_code_exec_popup_base;
use crate::ui::code_exec_popup_layout::{CodeExecPopupLayout, code_exec_popup_layout};
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::state::CodeExecHover;
use crate::framework::widget_system::context::{UpdateOutput, WidgetFrame};
use std::error::Error;

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
    pending: &'a crate::ui::state::PendingCodeExec,
    live: Option<&'a crate::ui::state::CodeExecLive>,
    reason_input: &'a mut tui_textarea::TextArea<'static>,
) -> crate::ui::code_exec_popup::CodeExecPopupParams<'a, 'static> {
    crate::ui::code_exec_popup::CodeExecPopupParams {
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
