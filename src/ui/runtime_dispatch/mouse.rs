use crate::ui::jump::{jump_row_at, jump_visible_rows};
use crate::ui::model_popup::{model_row_at, model_visible_rows};
use crate::ui::overlay::OverlayKind;
use crate::ui::prompt_popup::{prompt_row_at, prompt_visible_rows};
use crate::ui::runtime_events::handle_mouse_event;
use crate::ui::runtime_view::{apply_view_action, handle_view_mouse, ViewAction, ViewState};
use crate::ui::summary::summary_row_at;
use crossterm::event::{MouseEvent, MouseEventKind};

use super::{apply_model_selection, apply_prompt_selection, DispatchContext, LayoutContext};

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
        if view.overlay.is(OverlayKind::Jump) {
            let viewport_rows = jump_visible_rows(layout.msg_area);
            let max_scroll = jump_rows
                .len()
                .saturating_sub(viewport_rows)
                .max(1)
                .saturating_sub(1);
            match m.kind {
                MouseEventKind::ScrollUp => view.jump.scroll_by(-3, max_scroll, viewport_rows),
                MouseEventKind::ScrollDown => view.jump.scroll_by(3, max_scroll, viewport_rows),
                _ => {}
            }
        }
        if view.overlay.is(OverlayKind::Model) {
            let viewport_rows = model_visible_rows(layout.size, ctx.registry.models.len());
            let max_scroll = ctx
                .registry
                .models
                .len()
                .saturating_sub(viewport_rows)
                .max(1)
                .saturating_sub(1);
            match m.kind {
                MouseEventKind::ScrollUp => view.model.scroll_by(-3, max_scroll, viewport_rows),
                MouseEventKind::ScrollDown => view.model.scroll_by(3, max_scroll, viewport_rows),
                _ => {}
            }
        }
        if view.overlay.is(OverlayKind::Prompt) {
            let viewport_rows = prompt_visible_rows(layout.size, ctx.prompt_registry.prompts.len());
            let max_scroll = ctx
                .prompt_registry
                .prompts
                .len()
                .saturating_sub(viewport_rows)
                .max(1)
                .saturating_sub(1);
            match m.kind {
                MouseEventKind::ScrollUp => view.prompt.scroll_by(-3, max_scroll, viewport_rows),
                MouseEventKind::ScrollDown => view.prompt.scroll_by(3, max_scroll, viewport_rows),
                _ => {}
            }
        }
        let row = match view.overlay.active {
            Some(OverlayKind::Summary) => {
                summary_row_at(layout.msg_area, ctx.tabs.len(), m.column, m.row)
            }
            Some(OverlayKind::Jump) => jump_row_at(
                layout.msg_area,
                jump_rows.len(),
                m.column,
                m.row,
                view.jump.scroll,
            ),
            Some(OverlayKind::Model) => {
                model_row_at(
                    layout.size,
                    ctx.registry.models.len(),
                    view.model.scroll,
                    m.column,
                    m.row,
                )
            }
            Some(OverlayKind::Prompt) => prompt_row_at(
                layout.size,
                ctx.prompt_registry.prompts.len(),
                view.prompt.scroll,
                m.column,
                m.row,
            ),
            None => None,
        };
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
