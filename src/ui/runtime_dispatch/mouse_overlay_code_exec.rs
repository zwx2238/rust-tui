use crate::ui::code_exec_popup_layout::CodeExecPopupLayout;
use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use crate::ui::state::{CodeExecHover, CodeExecReasonTarget, PendingCommand};
use crossterm::event::MouseEvent;

use super::{apply_scroll, is_mouse_down, is_mouse_moved, point_in_rect, scroll_delta};

pub(crate) fn handle_code_exec_overlay_mouse(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    let theme = ctx.theme;
    let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) else {
        return handle_code_exec_fallback(m, ctx, layout, view);
    };
    let Some(pending) = tab_state.app.pending_code_exec.clone() else {
        return handle_code_exec_fallback(m, ctx, layout, view);
    };
    let popup = code_exec_popup_layout(layout.size, tab_state.app.code_exec_reason_target.is_some());
    if handle_code_exec_popup_mouse(m, theme, view, tab_state, &pending, popup) {
        return true;
    }
    handle_code_exec_fallback(m, ctx, layout, view)
}

fn handle_code_exec_popup_mouse(
    m: MouseEvent,
    theme: &crate::render::RenderTheme,
    view: &mut ViewState,
    tab_state: &mut TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
) -> bool {
    if handle_code_exec_hover(m, tab_state, popup) {
        return true;
    }
    if handle_code_exec_scroll(m, theme, tab_state, pending, popup) {
        return true;
    }
    if handle_code_exec_click(m, tab_state, view, popup) {
        return true;
    }
    false
}

fn handle_code_exec_hover(m: MouseEvent, tab_state: &mut TabState, popup: CodeExecPopupLayout) -> bool {
    if !is_mouse_moved(m.kind) {
        return false;
    }
    let reason_mode = tab_state.app.code_exec_reason_target.is_some();
    tab_state.app.code_exec_hover = hover_at(m, popup, reason_mode);
    true
}

fn hover_at(m: MouseEvent, popup: CodeExecPopupLayout, reason_mode: bool) -> Option<CodeExecHover> {
    if point_in_rect(m.column, m.row, popup.approve_btn) {
        Some(if reason_mode { CodeExecHover::ReasonConfirm } else { CodeExecHover::Approve })
    } else if point_in_rect(m.column, m.row, popup.deny_btn) {
        Some(if reason_mode { CodeExecHover::ReasonBack } else { CodeExecHover::Deny })
    } else if point_in_rect(m.column, m.row, popup.stop_btn) {
        Some(CodeExecHover::Stop)
    } else if point_in_rect(m.column, m.row, popup.exit_btn) {
        Some(CodeExecHover::Exit)
    } else {
        None
    }
}

fn handle_code_exec_scroll(
    m: MouseEvent,
    theme: &crate::render::RenderTheme,
    tab_state: &mut TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    let Some(delta) = scroll_delta(m.kind) else {
        return false;
    };
    if handle_code_exec_code_scroll(m, theme, tab_state, pending, popup, delta) {
        return true;
    }
    let (stdout, stderr) = code_exec_output(tab_state);
    if handle_code_exec_stdout_scroll(m, &stdout, theme, tab_state, popup.stdout_text_area, delta) {
        return true;
    }
    if handle_code_exec_stderr_scroll(m, &stderr, theme, tab_state, popup.stderr_text_area, delta) {
        return true;
    }
    true
}

fn handle_code_exec_code_scroll(
    m: MouseEvent,
    theme: &crate::render::RenderTheme,
    tab_state: &mut TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    delta: i32,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.code_text_area) {
        return false;
    }
    let max_scroll = code_max_scroll(
        &pending.code,
        popup.code_text_area.width,
        popup.code_text_area.height,
        theme,
    );
    apply_scroll(&mut tab_state.app.code_exec_scroll, delta, max_scroll);
    true
}

fn handle_code_exec_stdout_scroll(
    m: MouseEvent,
    content: &str,
    theme: &crate::render::RenderTheme,
    tab_state: &mut TabState,
    area: ratatui::layout::Rect,
    delta: i32,
) -> bool {
    if !point_in_rect(m.column, m.row, area) {
        return false;
    }
    let max_scroll = stdout_max_scroll(content, area.width, area.height, theme);
    apply_scroll(&mut tab_state.app.code_exec_stdout_scroll, delta, max_scroll);
    true
}

fn handle_code_exec_stderr_scroll(
    m: MouseEvent,
    content: &str,
    theme: &crate::render::RenderTheme,
    tab_state: &mut TabState,
    area: ratatui::layout::Rect,
    delta: i32,
) -> bool {
    if !point_in_rect(m.column, m.row, area) {
        return false;
    }
    let max_scroll = stderr_max_scroll(content, area.width, area.height, theme);
    apply_scroll(&mut tab_state.app.code_exec_stderr_scroll, delta, max_scroll);
    true
}

fn code_exec_output(tab_state: &TabState) -> (String, String) {
    tab_state
        .app
        .code_exec_live
        .as_ref()
        .and_then(|live| live.lock().ok().map(|live| (live.stdout.clone(), live.stderr.clone())))
        .unwrap_or_else(|| (String::new(), String::new()))
}

fn handle_code_exec_click(
    m: MouseEvent,
    tab_state: &mut TabState,
    view: &mut ViewState,
    popup: CodeExecPopupLayout,
) -> bool {
    if !is_mouse_down(m.kind) {
        return false;
    }
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    let finished = code_exec_finished(tab_state);
    let running = tab_state.app.code_exec_live.is_some() && !finished;
    if let Some(target) = tab_state.app.code_exec_reason_target {
        if handle_code_exec_reason_click(m, tab_state, view, popup, target) {
            return true;
        }
    }
    handle_code_exec_action_click(m, tab_state, view, popup, running, finished)
}

fn code_exec_finished(tab_state: &TabState) -> bool {
    tab_state
        .app
        .code_exec_live
        .as_ref()
        .and_then(|live| live.lock().ok().map(|live| live.done || live.exit_code.is_some()))
        .unwrap_or(false)
}

fn handle_code_exec_reason_click(
    m: MouseEvent,
    tab_state: &mut TabState,
    view: &mut ViewState,
    popup: CodeExecPopupLayout,
    target: CodeExecReasonTarget,
) -> bool {
    if point_in_rect(m.column, m.row, popup.approve_btn) {
        tab_state.app.pending_command = Some(match target {
            CodeExecReasonTarget::Deny => PendingCommand::DenyCodeExec,
            CodeExecReasonTarget::Stop => PendingCommand::StopCodeExec,
        });
        if matches!(target, CodeExecReasonTarget::Deny) {
            view.overlay.close();
        }
        return true;
    }
    if point_in_rect(m.column, m.row, popup.deny_btn) {
        clear_code_exec_reason(tab_state);
        return true;
    }
    false
}

fn handle_code_exec_action_click(
    m: MouseEvent,
    tab_state: &mut TabState,
    view: &mut ViewState,
    popup: CodeExecPopupLayout,
    running: bool,
    finished: bool,
) -> bool {
    if finished && point_in_rect(m.column, m.row, popup.exit_btn) {
        tab_state.app.pending_command = Some(PendingCommand::ExitCodeExec);
        return true;
    }
    if running && point_in_rect(m.column, m.row, popup.stop_btn) {
        tab_state.app.code_exec_reason_target = Some(CodeExecReasonTarget::Stop);
        reset_code_exec_reason_input(tab_state);
        return true;
    }
    if point_in_rect(m.column, m.row, popup.approve_btn) {
        tab_state.app.pending_command = Some(PendingCommand::ApproveCodeExec);
        tab_state.app.code_exec_hover = None;
        view.overlay.close();
        return true;
    }
    if point_in_rect(m.column, m.row, popup.deny_btn) {
        tab_state.app.code_exec_reason_target = Some(CodeExecReasonTarget::Deny);
        reset_code_exec_reason_input(tab_state);
        return true;
    }
    false
}

fn reset_code_exec_reason_input(tab_state: &mut TabState) {
    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
    tab_state.app.code_exec_hover = None;
}

fn clear_code_exec_reason(tab_state: &mut TabState) {
    tab_state.app.code_exec_reason_target = None;
    reset_code_exec_reason_input(tab_state);
}

fn handle_code_exec_fallback(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    if is_mouse_down(m.kind) {
        if point_in_rect(m.column, m.row, layout.tabs_area)
            || point_in_rect(m.column, m.row, layout.category_area)
        {
            return false;
        }
        view.overlay.close();
    }
    if is_mouse_moved(m.kind) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            tab_state.app.code_exec_hover = None;
        }
    }
    true
}
