use crate::ui::code_exec_popup_layout::code_exec_popup_layout;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};
use crate::ui::file_patch_popup_layout::file_patch_popup_layout;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext};
use crate::ui::runtime_view::ViewState;
use crate::ui::scroll::SCROLL_STEP_I32;
use crossterm::event::{MouseEvent, MouseEventKind};

pub(crate) fn handle_code_exec_overlay_mouse(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if let Some(pending) = tab_state.app.pending_code_exec.clone() {
            let popup = code_exec_popup_layout(
                layout.size,
                tab_state.app.code_exec_reason_target.is_some(),
            );
            let in_popup = point_in_rect(m.column, m.row, popup.popup);
            if matches!(m.kind, MouseEventKind::Moved) {
                let reason_mode = tab_state.app.code_exec_reason_target.is_some();
                tab_state.app.code_exec_hover = if point_in_rect(m.column, m.row, popup.approve_btn)
                {
                    Some(if reason_mode {
                        crate::ui::state::CodeExecHover::ReasonConfirm
                    } else {
                        crate::ui::state::CodeExecHover::Approve
                    })
                } else if point_in_rect(m.column, m.row, popup.deny_btn) {
                    Some(if reason_mode {
                        crate::ui::state::CodeExecHover::ReasonBack
                    } else {
                        crate::ui::state::CodeExecHover::Deny
                    })
                } else if point_in_rect(m.column, m.row, popup.stop_btn) {
                    Some(crate::ui::state::CodeExecHover::Stop)
                } else if point_in_rect(m.column, m.row, popup.exit_btn) {
                    Some(crate::ui::state::CodeExecHover::Exit)
                } else {
                    None
                };
                return true;
            }
            if in_popup
                && matches!(
                    m.kind,
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                )
            {
                let delta = match m.kind {
                    MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
                    MouseEventKind::ScrollDown => SCROLL_STEP_I32,
                    _ => 0,
                };
                if point_in_rect(m.column, m.row, popup.code_text_area) {
                    let max_scroll = code_max_scroll(
                        &pending.code,
                        popup.code_text_area.width,
                        popup.code_text_area.height,
                        ctx.theme,
                    );
                    let next = (tab_state.app.code_exec_scroll as i32 + delta).max(0) as usize;
                    tab_state.app.code_exec_scroll = next.min(max_scroll);
                    return true;
                }
                let (stdout, stderr) = tab_state
                    .app
                    .code_exec_live
                    .as_ref()
                    .and_then(|l| l.lock().ok().map(|l| (l.stdout.clone(), l.stderr.clone())))
                    .unwrap_or_else(|| (String::new(), String::new()));
                if point_in_rect(m.column, m.row, popup.stdout_text_area) {
                    let max_scroll = stdout_max_scroll(
                        &stdout,
                        popup.stdout_text_area.width,
                        popup.stdout_text_area.height,
                        ctx.theme,
                    );
                    let next =
                        (tab_state.app.code_exec_stdout_scroll as i32 + delta).max(0) as usize;
                    tab_state.app.code_exec_stdout_scroll = next.min(max_scroll);
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.stderr_text_area) {
                    let max_scroll = stderr_max_scroll(
                        &stderr,
                        popup.stderr_text_area.width,
                        popup.stderr_text_area.height,
                        ctx.theme,
                    );
                    let next =
                        (tab_state.app.code_exec_stderr_scroll as i32 + delta).max(0) as usize;
                    tab_state.app.code_exec_stderr_scroll = next.min(max_scroll);
                    return true;
                }
            }
            if in_popup && matches!(m.kind, MouseEventKind::Down(_)) {
                let finished = tab_state
                    .app
                    .code_exec_live
                    .as_ref()
                    .and_then(|l| l.lock().ok().map(|l| l.done || l.exit_code.is_some()))
                    .unwrap_or(false);
                let running = tab_state.app.code_exec_live.is_some() && !finished;
                let reason_target = tab_state.app.code_exec_reason_target;
                if let Some(target) = reason_target {
                    if point_in_rect(m.column, m.row, popup.approve_btn) {
                        tab_state.app.pending_command = Some(match target {
                            crate::ui::state::CodeExecReasonTarget::Deny => {
                                crate::ui::state::PendingCommand::DenyCodeExec
                            }
                            crate::ui::state::CodeExecReasonTarget::Stop => {
                                crate::ui::state::PendingCommand::StopCodeExec
                            }
                        });
                        if matches!(target, crate::ui::state::CodeExecReasonTarget::Deny) {
                            view.overlay.close();
                        }
                        return true;
                    }
                    if point_in_rect(m.column, m.row, popup.deny_btn) {
                        tab_state.app.code_exec_reason_target = None;
                        tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
                        tab_state.app.code_exec_hover = None;
                        return true;
                    }
                }
                if finished && point_in_rect(m.column, m.row, popup.exit_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::ExitCodeExec);
                    return true;
                }
                if running && point_in_rect(m.column, m.row, popup.stop_btn) {
                    tab_state.app.code_exec_reason_target =
                        Some(crate::ui::state::CodeExecReasonTarget::Stop);
                    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
                    tab_state.app.code_exec_hover = None;
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.approve_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::ApproveCodeExec);
                    tab_state.app.code_exec_hover = None;
                    view.overlay.close();
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.deny_btn) {
                    tab_state.app.code_exec_reason_target =
                        Some(crate::ui::state::CodeExecReasonTarget::Deny);
                    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
                    tab_state.app.code_exec_hover = None;
                    return true;
                }
            }
        }
    }
    if matches!(m.kind, MouseEventKind::Down(_)) {
        view.overlay.close();
    }
    if matches!(m.kind, MouseEventKind::Moved) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            tab_state.app.code_exec_hover = None;
        }
    }
    true
}

pub(crate) fn handle_file_patch_overlay_mouse(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if let Some(pending) = tab_state.app.pending_file_patch.clone() {
            let popup = file_patch_popup_layout(layout.size);
            let in_popup = point_in_rect(m.column, m.row, popup.popup);
            if matches!(m.kind, MouseEventKind::Moved) {
                tab_state.app.file_patch_hover = if point_in_rect(m.column, m.row, popup.apply_btn)
                {
                    Some(crate::ui::state::FilePatchHover::Apply)
                } else if point_in_rect(m.column, m.row, popup.cancel_btn) {
                    Some(crate::ui::state::FilePatchHover::Cancel)
                } else {
                    None
                };
                return true;
            }
            if in_popup
                && matches!(
                    m.kind,
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                )
            {
                let delta = match m.kind {
                    MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
                    MouseEventKind::ScrollDown => SCROLL_STEP_I32,
                    _ => 0,
                };
                let max_scroll = patch_max_scroll(
                    &pending.preview,
                    popup.preview_area.width,
                    popup.preview_area.height,
                    ctx.theme,
                );
                let next = (tab_state.app.file_patch_scroll as i32 + delta).max(0) as usize;
                tab_state.app.file_patch_scroll = next.min(max_scroll);
                return true;
            }
            if in_popup && matches!(m.kind, MouseEventKind::Down(_)) {
                if point_in_rect(m.column, m.row, popup.apply_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::ApplyFilePatch);
                    tab_state.app.file_patch_hover = None;
                    view.overlay.close();
                    return true;
                }
                if point_in_rect(m.column, m.row, popup.cancel_btn) {
                    tab_state.app.pending_command =
                        Some(crate::ui::state::PendingCommand::CancelFilePatch);
                    tab_state.app.file_patch_hover = None;
                    view.overlay.close();
                    return true;
                }
            }
        }
    }
    if matches!(m.kind, MouseEventKind::Down(_)) {
        view.overlay.close();
    }
    if matches!(m.kind, MouseEventKind::Moved) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            tab_state.app.file_patch_hover = None;
        }
    }
    true
}

fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}
