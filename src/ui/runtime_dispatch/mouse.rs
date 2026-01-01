use crate::ui::jump::jump_row_at;
use crate::ui::model_popup::model_row_at;
use crate::ui::overlay::OverlayKind;
use crate::ui::prompt_popup::{prompt_row_at, prompt_visible_rows};
use crate::ui::runtime_events::handle_mouse_event;
use crate::ui::runtime_view::{apply_view_action, handle_view_mouse, ViewAction, ViewState};
use crate::ui::summary::summary_row_at;
use crossterm::event::{MouseEvent, MouseEventKind};

use super::{can_change_prompt, DispatchContext, LayoutContext};

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
            if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
                if let Some(model) = ctx.registry.models.get(idx) {
                    tab_state.app.model_key = model.key.clone();
                }
            }
            return;
        }
        if let ViewAction::SelectPrompt(idx) = action {
            if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
                if can_change_prompt(&tab_state.app) {
                    if let Some(prompt) = ctx.prompt_registry.prompts.get(idx) {
                        tab_state
                            .app
                            .set_system_prompt(&prompt.key, &prompt.content);
                    }
                } else {
                    tab_state.app.messages.push(crate::types::Message {
                        role: "assistant".to_string(),
                        content: "已开始对话，无法切换系统提示词，请新开 tab。".to_string(),
                    });
                }
            }
            return;
        }
        let _ = apply_view_action(action, jump_rows, ctx.tabs, ctx.active_tab);
    }
}
