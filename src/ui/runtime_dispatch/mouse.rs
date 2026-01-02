use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
use crate::ui::runtime_events::{handle_mouse_event, handle_tab_category_click};
use crate::ui::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_mouse};
use crate::ui::scroll::SCROLL_STEP_I32;
use crossterm::event::{MouseEvent, MouseEventKind};

use super::mouse_overlay::{handle_code_exec_overlay_mouse, handle_file_patch_overlay_mouse};
use super::{DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection};

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
        help: crate::ui::shortcut_help::help_rows_len(),
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
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if crate::ui::command_suggestions::handle_command_suggestion_click(
            &mut tab_state.app,
            layout.msg_area,
            layout.input_area,
            m.column,
            m.row,
        ) {
            return;
        }
    }
    if matches!(m.kind, MouseEventKind::Down(_)) {
        if handle_tab_category_click(
            m.column,
            m.row,
            ctx.tabs,
            ctx.active_tab,
            ctx.categories,
            ctx.active_category,
            layout.tabs_area,
            layout.category_area,
        ) {
            return;
        }
    }
    if view.overlay.is(crate::ui::overlay::OverlayKind::CodeExec) {
        if handle_code_exec_overlay_mouse(m, ctx, layout, view) {
            return;
        }
    }
    if view.overlay.is(crate::ui::overlay::OverlayKind::FilePatch) {
        if handle_file_patch_overlay_mouse(m, ctx, layout, view) {
            return;
        }
    }
    if view.is_chat() {
        if let Some(msg_idx) = handle_mouse_event(
            m,
            ctx.tabs,
            ctx.active_tab,
            ctx.categories,
            ctx.active_category,
            layout.tabs_area,
            layout.msg_area,
            layout.input_area,
            layout.category_area,
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
        let _ = apply_view_action(
            action,
            jump_rows,
            ctx.tabs,
            ctx.active_tab,
            ctx.categories,
            ctx.active_category,
        );
    }
}
