use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
use crate::ui::runtime_events::handle_mouse_event;
use crate::ui::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_mouse};
use crate::ui::scroll::SCROLL_STEP_I32;
use crossterm::event::{MouseEvent, MouseEventKind};

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
    if view.is_chat() {
        handle_mouse_event(
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
        );
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
