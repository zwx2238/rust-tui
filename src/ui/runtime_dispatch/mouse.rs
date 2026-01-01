use crate::ui::jump::{jump_row_at, jump_visible_rows};
use crate::ui::model_popup::{model_row_at, model_visible_rows};
use crate::ui::overlay::OverlayKind;
use crate::ui::popup_table::popup_row_at;
use crate::ui::prompt_popup::{prompt_row_at, prompt_visible_rows};
use crate::ui::runtime_events::handle_mouse_event;
use crate::ui::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_mouse};
use crate::ui::scroll::{SCROLL_STEP_I32, max_scroll};
use crossterm::event::{MouseEvent, MouseEventKind};

use super::{DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection};

fn scroll_selection(
    selection: &mut crate::ui::selection_state::SelectionState,
    delta: i32,
    len: usize,
    viewport_rows: usize,
) {
    let max_scroll = max_scroll(len, viewport_rows);
    selection.scroll_by(delta, max_scroll, viewport_rows);
}

fn overlay_row_at(
    view: &ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
    jump_rows: &[crate::ui::jump::JumpRow],
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    match view.overlay.active {
        Some(OverlayKind::Summary) => {
            popup_row_at(layout.msg_area, ctx.tabs.len(), 0, mouse_x, mouse_y)
        }
        Some(OverlayKind::Jump) => jump_row_at(
            layout.msg_area,
            jump_rows.len(),
            mouse_x,
            mouse_y,
            view.jump.scroll,
        ),
        Some(OverlayKind::Model) => model_row_at(
            layout.size,
            ctx.registry.models.len(),
            view.model.scroll,
            mouse_x,
            mouse_y,
        ),
        Some(OverlayKind::Prompt) => prompt_row_at(
            layout.size,
            ctx.prompt_registry.prompts.len(),
            view.prompt.scroll,
            mouse_x,
            mouse_y,
        ),
        None => None,
    }
}

fn handle_overlay_scroll(
    view: &mut ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
    jump_rows: &[crate::ui::jump::JumpRow],
    kind: MouseEventKind,
) {
    if view.overlay.is(OverlayKind::Jump) {
        let viewport_rows = jump_visible_rows(layout.msg_area);
        match kind {
            MouseEventKind::ScrollUp => scroll_selection(
                &mut view.jump,
                -SCROLL_STEP_I32,
                jump_rows.len(),
                viewport_rows,
            ),
            MouseEventKind::ScrollDown => scroll_selection(
                &mut view.jump,
                SCROLL_STEP_I32,
                jump_rows.len(),
                viewport_rows,
            ),
            _ => {}
        }
    }
    if view.overlay.is(OverlayKind::Model) {
        let viewport_rows = model_visible_rows(layout.size, ctx.registry.models.len());
        match kind {
            MouseEventKind::ScrollUp => scroll_selection(
                &mut view.model,
                -SCROLL_STEP_I32,
                ctx.registry.models.len(),
                viewport_rows,
            ),
            MouseEventKind::ScrollDown => scroll_selection(
                &mut view.model,
                SCROLL_STEP_I32,
                ctx.registry.models.len(),
                viewport_rows,
            ),
            _ => {}
        }
    }
    if view.overlay.is(OverlayKind::Prompt) {
        let viewport_rows = prompt_visible_rows(layout.size, ctx.prompt_registry.prompts.len());
        match kind {
            MouseEventKind::ScrollUp => scroll_selection(
                &mut view.prompt,
                -SCROLL_STEP_I32,
                ctx.prompt_registry.prompts.len(),
                viewport_rows,
            ),
            MouseEventKind::ScrollDown => scroll_selection(
                &mut view.prompt,
                SCROLL_STEP_I32,
                ctx.prompt_registry.prompts.len(),
                viewport_rows,
            ),
            _ => {}
        }
    }
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
        handle_overlay_scroll(view, ctx, layout, jump_rows, m.kind);
        let row = overlay_row_at(view, ctx, layout, jump_rows, m.column, m.row);
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
