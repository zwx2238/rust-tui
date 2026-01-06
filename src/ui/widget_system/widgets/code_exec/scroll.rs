use crate::ui::code_exec_popup_layout::CodeExecPopupLayout;
use crate::ui::code_exec_popup_text::{code_max_scroll, stderr_max_scroll, stdout_max_scroll};

use super::helpers::code_exec_output;
use super::helpers::{apply_scroll, point_in_rect};

pub(super) fn handle_code_exec_scroll(
    m: crossterm::event::MouseEvent,
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    delta: i32,
) -> bool {
    if !point_in_rect(m.column, m.row, popup.popup) {
        return false;
    }
    if point_in_rect(m.column, m.row, popup.code_text_area) {
        return scroll_code(tab_state, pending, popup, theme, delta);
    }
    let (stdout, stderr) = code_exec_output(tab_state);
    if point_in_rect(m.column, m.row, popup.stdout_text_area) {
        return scroll_stdout(tab_state, &stdout, popup, theme, delta);
    }
    if point_in_rect(m.column, m.row, popup.stderr_text_area) {
        return scroll_stderr(tab_state, &stderr, popup, theme, delta);
    }
    false
}

fn scroll_code(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    delta: i32,
) -> bool {
    let max_scroll = code_max_scroll(
        &pending.code,
        popup.code_text_area.width,
        popup.code_text_area.height,
        theme,
    );
    apply_scroll(&mut tab_state.app.code_exec_scroll, delta, max_scroll);
    true
}

fn scroll_stdout(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    stdout: &str,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    delta: i32,
) -> bool {
    let max_scroll = stdout_max_scroll(
        stdout,
        popup.stdout_text_area.width,
        popup.stdout_text_area.height,
        theme,
    );
    apply_scroll(
        &mut tab_state.app.code_exec_stdout_scroll,
        delta,
        max_scroll,
    );
    true
}

fn scroll_stderr(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    stderr: &str,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    delta: i32,
) -> bool {
    let max_scroll = stderr_max_scroll(
        stderr,
        popup.stderr_text_area.width,
        popup.stderr_text_area.height,
        theme,
    );
    apply_scroll(
        &mut tab_state.app.code_exec_stderr_scroll,
        delta,
        max_scroll,
    );
    true
}
