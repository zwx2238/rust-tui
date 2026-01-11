use super::popup_layout::{CodeExecPopupLayout, code_exec_popup_layout};
use super::popup_text::{
    build_code_text, build_stderr_text, build_stdout_text, code_plain_lines, stderr_plain_lines,
    stdout_plain_lines,
};
use crate::framework::widget_system::interaction::selection::{Selection, chat_position_from_mouse, extract_selection};
use crate::framework::widget_system::runtime::state::CodeExecSelectionTarget;

use super::helpers::{code_exec_output, point_in_rect};

pub(super) fn handle_code_exec_selection_start(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    if point_in_rect(m.column, m.row, popup.code_text_area) {
        return start_code_selection(tab_state, pending, popup, theme, m);
    }
    let (stdout, stderr) = code_exec_output(tab_state);
    if point_in_rect(m.column, m.row, popup.stdout_text_area) {
        return start_stdout_selection(tab_state, &stdout, popup, theme, m);
    }
    if point_in_rect(m.column, m.row, popup.stderr_text_area) {
        return start_stderr_selection(tab_state, &stderr, popup, theme, m);
    }
    false
}

pub(super) fn handle_code_exec_selection_drag(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    let Some(target) = tab_state.app.code_exec_selecting else {
        return false;
    };
    let (stdout, stderr) = code_exec_output(tab_state);
    match target {
        CodeExecSelectionTarget::Code => {
            update_code_drag(tab_state, pending, popup, theme, m);
        }
        CodeExecSelectionTarget::Stdout => {
            update_stdout_drag(tab_state, &stdout, popup, theme, m);
        }
        CodeExecSelectionTarget::Stderr => {
            update_stderr_drag(tab_state, &stderr, popup, theme, m);
        }
    }
    true
}

pub(super) fn clear_code_exec_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
) -> bool {
    if tab_state.app.code_exec_selecting.is_none() {
        return false;
    }
    tab_state.app.code_exec_selecting = None;
    true
}

pub(super) fn copy_code_exec_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingCodeExec,
    layout: &crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout,
    theme: &crate::render::RenderTheme,
) -> bool {
    let popup = code_exec_popup_layout(
        layout.size,
        tab_state.app.code_exec_reason_target.is_some(),
    );
    if let Some(selection) = tab_state.app.code_exec_code_selection {
        let lines = code_plain_lines(&pending.code, popup.code_text_area.width, theme);
        return copy_selection_text(lines, selection);
    }
    let (stdout, stderr) = code_exec_output(tab_state);
    if let Some(selection) = tab_state.app.code_exec_stdout_selection {
        let lines = stdout_plain_lines(&stdout, popup.stdout_text_area.width, theme);
        return copy_selection_text(lines, selection);
    }
    if let Some(selection) = tab_state.app.code_exec_stderr_selection {
        let lines = stderr_plain_lines(&stderr, popup.stderr_text_area.width, theme);
        return copy_selection_text(lines, selection);
    }
    false
}

fn start_code_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    let (text, _) = build_code_text(
        &pending.code,
        popup.code_text_area.width,
        popup.code_text_area.height,
        tab_state.app.code_exec_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.code_exec_scroll,
        popup.code_text_area,
        m,
    );
    start_code_exec_selection(tab_state, CodeExecSelectionTarget::Code, pos);
    true
}

fn start_stdout_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    stdout: &str,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    let (text, _) = build_stdout_text(
        Some(stdout),
        popup.stdout_text_area.width,
        popup.stdout_text_area.height,
        tab_state.app.code_exec_stdout_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.code_exec_stdout_scroll,
        popup.stdout_text_area,
        m,
    );
    start_code_exec_selection(tab_state, CodeExecSelectionTarget::Stdout, pos);
    true
}

fn start_stderr_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    stderr: &str,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) -> bool {
    let (text, _) = build_stderr_text(
        Some(stderr),
        popup.stderr_text_area.width,
        popup.stderr_text_area.height,
        tab_state.app.code_exec_stderr_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.code_exec_stderr_scroll,
        popup.stderr_text_area,
        m,
    );
    start_code_exec_selection(tab_state, CodeExecSelectionTarget::Stderr, pos);
    true
}

fn update_code_drag(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    pending: &crate::framework::widget_system::runtime::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) {
    let (text, _) = build_code_text(
        &pending.code,
        popup.code_text_area.width,
        popup.code_text_area.height,
        tab_state.app.code_exec_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.code_exec_scroll,
        popup.code_text_area,
        m,
    );
    update_selection(&mut tab_state.app.code_exec_code_selection, pos);
}

fn update_stdout_drag(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    stdout: &str,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) {
    let (text, _) = build_stdout_text(
        Some(stdout),
        popup.stdout_text_area.width,
        popup.stdout_text_area.height,
        tab_state.app.code_exec_stdout_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.code_exec_stdout_scroll,
        popup.stdout_text_area,
        m,
    );
    update_selection(&mut tab_state.app.code_exec_stdout_selection, pos);
}

fn update_stderr_drag(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    stderr: &str,
    popup: CodeExecPopupLayout,
    theme: &crate::render::RenderTheme,
    m: crossterm::event::MouseEvent,
) {
    let (text, _) = build_stderr_text(
        Some(stderr),
        popup.stderr_text_area.width,
        popup.stderr_text_area.height,
        tab_state.app.code_exec_stderr_scroll,
        theme,
    );
    let pos = selection_position_for_panel(
        &text,
        tab_state.app.code_exec_stderr_scroll,
        popup.stderr_text_area,
        m,
    );
    update_selection(&mut tab_state.app.code_exec_stderr_selection, pos);
}

fn update_selection(target: &mut Option<Selection>, pos: (usize, usize)) {
    let next = match *target {
        Some(existing) => Selection {
            start: existing.start,
            end: pos,
        },
        None => Selection {
            start: pos,
            end: pos,
        },
    };
    *target = Some(next);
}

fn start_code_exec_selection(
    tab_state: &mut crate::framework::widget_system::runtime::runtime_helpers::TabState,
    target: CodeExecSelectionTarget,
    pos: (usize, usize),
) {
    tab_state.app.code_exec_selecting = Some(target);
    let selection = Some(Selection {
        start: pos,
        end: pos,
    });
    match target {
        CodeExecSelectionTarget::Code => {
            tab_state.app.code_exec_code_selection = selection;
            tab_state.app.code_exec_stdout_selection = None;
            tab_state.app.code_exec_stderr_selection = None;
        }
        CodeExecSelectionTarget::Stdout => {
            tab_state.app.code_exec_stdout_selection = selection;
            tab_state.app.code_exec_code_selection = None;
            tab_state.app.code_exec_stderr_selection = None;
        }
        CodeExecSelectionTarget::Stderr => {
            tab_state.app.code_exec_stderr_selection = selection;
            tab_state.app.code_exec_code_selection = None;
            tab_state.app.code_exec_stdout_selection = None;
        }
    }
}

fn selection_position_for_panel(
    text: &ratatui::text::Text<'static>,
    scroll: usize,
    area: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> (usize, usize) {
    let scroll_u16 = scroll.min(u16::MAX as usize) as u16;
    chat_position_from_mouse(text, scroll_u16, area, m.column, m.row)
}

fn copy_selection_text(lines: Vec<String>, selection: Selection) -> bool {
    let text = extract_selection(&lines, selection);
    if !text.is_empty() {
        crate::framework::widget_system::interaction::clipboard::set(&text);
    }
    true
}
