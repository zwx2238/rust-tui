use crate::ui::jump::jump_row_at;
use crate::ui::model_popup::model_row_at;
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
            match m.kind {
                MouseEventKind::ScrollUp => {
                    view.jump.scroll = view.jump.scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    view.jump.scroll = view.jump.scroll.saturating_add(3);
                }
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
                MouseEventKind::ScrollUp => {
                    view.prompt.scroll = view.prompt.scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    view.prompt.scroll = view.prompt.scroll.saturating_add(3);
                }
                _ => {}
            }
            view.prompt.scroll = view.prompt.scroll.min(max_scroll);
            view.prompt.ensure_visible(viewport_rows);
        }
        let row = match view.overlay.active {
            Some(OverlayKind::Summary) => summary_row_at(layout.msg_area, ctx.tabs.len(), m.row),
            Some(OverlayKind::Jump) => {
                jump_row_at(layout.msg_area, jump_rows.len(), m.row, view.jump.scroll)
            }
            Some(OverlayKind::Model) => {
                model_row_at(layout.size, ctx.registry.models.len(), m.column, m.row)
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
