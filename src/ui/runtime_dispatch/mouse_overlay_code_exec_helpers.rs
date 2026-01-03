use crate::ui::code_exec_popup_layout::CodeExecPopupLayout;
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::CodeExecHover;
use crossterm::event::MouseEvent;

use super::point_in_rect;

pub(super) fn hover_at(
    m: MouseEvent,
    popup: CodeExecPopupLayout,
    reason_mode: bool,
) -> Option<CodeExecHover> {
    if point_in_rect(m.column, m.row, popup.approve_btn) {
        Some(if reason_mode {
            CodeExecHover::ReasonConfirm
        } else {
            CodeExecHover::Approve
        })
    } else if point_in_rect(m.column, m.row, popup.deny_btn) {
        Some(if reason_mode {
            CodeExecHover::ReasonBack
        } else {
            CodeExecHover::Deny
        })
    } else if point_in_rect(m.column, m.row, popup.stop_btn) {
        Some(CodeExecHover::Stop)
    } else if point_in_rect(m.column, m.row, popup.exit_btn) {
        Some(CodeExecHover::Exit)
    } else {
        None
    }
}

pub(super) fn code_exec_output(tab_state: &TabState) -> (String, String) {
    tab_state
        .app
        .code_exec_live
        .as_ref()
        .and_then(|live| {
            live.lock()
                .ok()
                .map(|live| (live.stdout.clone(), live.stderr.clone()))
        })
        .unwrap_or_else(|| (String::new(), String::new()))
}

pub(super) fn code_exec_finished(tab_state: &TabState) -> bool {
    tab_state
        .app
        .code_exec_live
        .as_ref()
        .and_then(|live| {
            live.lock()
                .ok()
                .map(|live| live.done || live.exit_code.is_some())
        })
        .unwrap_or(false)
}

pub(super) fn reset_code_exec_reason_input(tab_state: &mut TabState) {
    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
    tab_state.app.code_exec_hover = None;
}

pub(super) fn clear_code_exec_reason(tab_state: &mut TabState) {
    tab_state.app.code_exec_reason_target = None;
    reset_code_exec_reason_input(tab_state);
}
