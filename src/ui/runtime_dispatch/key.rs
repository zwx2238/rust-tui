use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_view::{apply_view_action, handle_view_key, ViewAction, ViewState};
use crate::ui::runtime_events::handle_key_event;
use crossterm::event::KeyEvent;

use super::{
    apply_model_selection, apply_prompt_selection, can_change_prompt, cycle_model, DispatchContext,
    LayoutContext, push_prompt_locked, sync_model_selection, sync_prompt_selection, new_tab,
    close_tab, prev_tab, next_tab,
};

pub(crate) fn handle_key_event_loop(
    key: KeyEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    if key.code == crossterm::event::KeyCode::F(5) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            if !can_change_prompt(&tab_state.app) {
                push_prompt_locked(tab_state);
                return Ok(false);
            }
        }
    }
    let action = handle_view_key(
        view,
        key,
        ctx.tabs.len(),
        jump_rows.len(),
        *ctx.active_tab,
    );
    if matches!(action, ViewAction::CycleModel) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            cycle_model(ctx.registry, &mut tab_state.app.model_key);
        }
        return Ok(false);
    }
    if key.code == crossterm::event::KeyCode::F(5) && view.overlay.is(OverlayKind::Prompt) {
        sync_prompt_selection(view, ctx, layout);
        return Ok(false);
    }
    if let ViewAction::SelectModel(idx) = action {
        apply_model_selection(ctx, idx);
        return Ok(false);
    }
    if let ViewAction::SelectPrompt(idx) = action {
        apply_prompt_selection(ctx, idx);
        return Ok(false);
    }
    if apply_view_action(action, jump_rows, ctx.tabs, ctx.active_tab) {
        return Ok(false);
    }
    if key.code == crossterm::event::KeyCode::F(4) && view.overlay.is(OverlayKind::Model) {
        sync_model_selection(view, ctx, layout);
        return Ok(false);
    }
    if !view.is_chat() {
        return Ok(false);
    }
    if key
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL)
    {
        match key.code {
            crossterm::event::KeyCode::Char('t') => {
                new_tab(ctx);
                return Ok(false);
            }
            crossterm::event::KeyCode::Char('w') => {
                close_tab(ctx);
                return Ok(false);
            }
            _ => {}
        }
    }
    match key.code {
        crossterm::event::KeyCode::F(8) => {
            prev_tab(ctx);
            return Ok(false);
        }
        crossterm::event::KeyCode::F(9) => {
            next_tab(ctx);
            return Ok(false);
        }
        _ => {}
    }
    if handle_key_event(
        key,
        ctx.tabs,
        *ctx.active_tab,
        ctx.last_session_id,
        ctx.msg_width,
        ctx.theme,
    )? {
        return Ok(true);
    }
    Ok(false)
}
