use crate::ui::code_exec_popup::{code_exec_max_scroll, code_exec_popup_layout, output_max_scroll};
use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
use crate::ui::runtime_events::handle_mouse_event;
use crate::ui::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_mouse};
use crate::ui::scroll::SCROLL_STEP_I32;
use crossterm::event::{MouseEvent, MouseEventKind};

use super::{
    DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection,
};

fn overlay_areas(layout: LayoutContext) -> OverlayAreas {
    OverlayAreas {
        full: layout.size,
        msg: layout.msg_area,
    }
}

fn overlay_counts(ctx: &DispatchContext<'_>, jump_rows: usize) -> OverlayRowCounts {
    OverlayRowCounts {
        tabs: ctx.tabs.len(),
        jump: jump_rows,
        models: ctx.registry.models.len(),
        prompts: ctx.prompt_registry.prompts.len(),
    }
}

fn overlay_row_at(
    view: &mut ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
    jump_rows: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    let areas = overlay_areas(layout);
    let counts = overlay_counts(ctx, jump_rows);
    with_active_table_handle(view, areas, counts, |handle| {
        handle.row_at(mouse_x, mouse_y)
    })
    .flatten()
}

fn handle_overlay_scroll(
    view: &mut ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
    jump_rows: usize,
    kind: MouseEventKind,
) {
    let delta = match kind {
        MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
        MouseEventKind::ScrollDown => SCROLL_STEP_I32,
        _ => return,
    };
    let areas = overlay_areas(layout);
    let counts = overlay_counts(ctx, jump_rows);
    let _ = with_active_table_handle(view, areas, counts, |mut handle| {
        handle.scroll_by(delta);
    });
}

pub(crate) fn handle_mouse_event_loop(
    m: MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
) {
    if view.overlay.is(crate::ui::overlay::OverlayKind::CodeExec) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            if let Some(pending) = tab_state.app.pending_code_exec.clone() {
                let popup = code_exec_popup_layout(layout.size);
                let in_popup = point_in_rect(m.column, m.row, popup.popup);
                if matches!(m.kind, MouseEventKind::Moved) {
                    tab_state.app.code_exec_hover = if point_in_rect(m.column, m.row, popup.approve_btn) {
                        Some(crate::ui::state::CodeExecHover::Approve)
                    } else if point_in_rect(m.column, m.row, popup.deny_btn) {
                        Some(crate::ui::state::CodeExecHover::Deny)
                    } else if point_in_rect(m.column, m.row, popup.exit_btn) {
                        Some(crate::ui::state::CodeExecHover::Exit)
                    } else {
                        None
                    };
                    return;
                }
                if in_popup && matches!(m.kind, MouseEventKind::ScrollUp | MouseEventKind::ScrollDown)
                {
                    let delta = match m.kind {
                        MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
                        MouseEventKind::ScrollDown => SCROLL_STEP_I32,
                        _ => 0,
                    };
                    if point_in_rect(m.column, m.row, popup.code_text_area) {
                        let max_scroll = code_exec_max_scroll(
                            &pending.code,
                            popup.code_text_area.width,
                            popup.code_text_area.height,
                            ctx.theme,
                        );
                        let next = (tab_state.app.code_exec_scroll as i32 + delta)
                            .max(0) as usize;
                        tab_state.app.code_exec_scroll = next.min(max_scroll);
                        return;
                    }
                    if point_in_rect(m.column, m.row, popup.output_text_area) {
                        let (stdout, stderr) = tab_state
                            .app
                            .code_exec_live
                            .as_ref()
                            .and_then(|l| l.lock().ok().map(|l| (l.stdout.clone(), l.stderr.clone())))
                            .unwrap_or_else(|| (String::new(), String::new()));
                        let max_scroll = output_max_scroll(
                            &stdout,
                            &stderr,
                            popup.output_text_area.width,
                            popup.output_text_area.height,
                            ctx.theme,
                        );
                        let next = (tab_state.app.code_exec_output_scroll as i32 + delta)
                            .max(0) as usize;
                        tab_state.app.code_exec_output_scroll = next.min(max_scroll);
                        return;
                    }
                }
                if in_popup && matches!(m.kind, MouseEventKind::Down(_)) {
                    let finished = tab_state
                        .app
                        .code_exec_live
                        .as_ref()
                        .and_then(|l| l.lock().ok().map(|l| l.done))
                        .unwrap_or(false);
                    if finished && point_in_rect(m.column, m.row, popup.exit_btn) {
                        tab_state.app.pending_code_exec = None;
                        tab_state.app.code_exec_live = None;
                        tab_state.app.code_exec_result_pushed = false;
                        tab_state.app.code_exec_hover = None;
                        tab_state.app.code_exec_scroll = 0;
                        tab_state.app.code_exec_output_scroll = 0;
                        view.overlay.close();
                        return;
                    }
                    if point_in_rect(m.column, m.row, popup.approve_btn) {
                        tab_state.app.pending_command =
                            Some(crate::ui::state::PendingCommand::ApproveCodeExec);
                        tab_state.app.code_exec_hover = None;
                        view.overlay.close();
                        return;
                    }
                    if point_in_rect(m.column, m.row, popup.deny_btn) {
                        tab_state.app.pending_command =
                            Some(crate::ui::state::PendingCommand::DenyCodeExec);
                        tab_state.app.code_exec_hover = None;
                        view.overlay.close();
                        return;
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
    }
    if view.is_chat() {
        if let Some(msg_idx) = handle_mouse_event(
            m,
            ctx.tabs,
            ctx.active_tab,
            layout.tabs_area,
            layout.msg_area,
            layout.input_area,
            ctx.msg_width,
            layout.view_height,
            layout.total_lines,
            ctx.theme,
        ) {
            let _ = super::fork_message_by_index(ctx, msg_idx);
        }
    } else {
        handle_overlay_scroll(view, ctx, layout, jump_rows.len(), m.kind);
        let row = overlay_row_at(view, ctx, layout, jump_rows.len(), m.column, m.row);
        let action = handle_view_mouse(view, row, ctx.tabs.len(), jump_rows.len(), m.kind);
        if let ViewAction::SelectModel(idx) = action {
            apply_model_selection(ctx, idx);
            return;
        }
        if let ViewAction::SelectPrompt(idx) = action {
            apply_prompt_selection(ctx, idx);
            return;
        }
        let _ = apply_view_action(action, jump_rows, ctx.tabs, ctx.active_tab);
    }
}

fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}
