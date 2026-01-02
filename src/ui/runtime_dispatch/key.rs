use crate::ui::overlay::OverlayKind;
use crate::ui::logic::stop_stream;
use crate::ui::runtime_events::handle_key_event;
use crate::ui::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_key};
use crossterm::event::KeyEvent;

use super::{
    DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection,
    can_change_prompt, close_all_tabs, close_other_tabs, close_tab, cycle_model, handle_nav_key,
    new_tab, next_tab, prev_tab, push_prompt_locked, sync_model_selection, sync_prompt_selection,
};

pub(crate) fn handle_key_event_loop(
    key: KeyEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        && key.code == crossterm::event::KeyCode::Char('q')
    {
        return Ok(true);
    }
    if handle_global_shortcuts(ctx, key) {
        return Ok(false);
    }
    if key.code == crossterm::event::KeyCode::F(6) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::SHIFT)
            {
                crate::ui::runtime_helpers::stop_and_edit(tab_state);
            } else {
                stop_stream(&mut tab_state.app);
            }
        }
        return Ok(false);
    }
    if key.code == crossterm::event::KeyCode::F(5) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            if !can_change_prompt(&tab_state.app) {
                push_prompt_locked(tab_state);
                return Ok(false);
            }
        }
    }
    if view.is_chat() {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            if handle_nav_key(&mut tab_state.app, key) {
                return Ok(false);
            }
        }
    }
    if view.overlay.is(OverlayKind::CodeExec) {
        handle_code_exec_overlay_key(ctx, view);
    }
    let action = handle_view_key(view, key, ctx.tabs.len(), jump_rows.len(), *ctx.active_tab);
    if matches!(action, ViewAction::CycleModel) {
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
            cycle_model(ctx.registry, &mut tab_state.app.model_key);
        }
        return Ok(false);
    }
    if let ViewAction::ForkMessage(idx) = action {
        if crate::ui::runtime_dispatch::fork_message_into_new_tab(ctx, jump_rows, idx) {
            view.overlay.close();
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
    if handle_key_event(
        key,
        ctx.tabs,
        *ctx.active_tab,
        ctx.msg_width,
        ctx.theme,
    )? {
        return Ok(true);
    }
    Ok(false)
}

fn handle_code_exec_overlay_key(ctx: &mut DispatchContext<'_>, view: &mut ViewState) {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if tab_state.app.pending_code_exec.is_none() {
            view.overlay.close();
        }
    }
}

fn handle_global_shortcuts(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if key
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL)
    {
        if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)
            && key.code == crossterm::event::KeyCode::Char('w')
        {
            close_all_tabs(ctx);
            return true;
        }
        match key.code {
            crossterm::event::KeyCode::Char('o') => {
                close_other_tabs(ctx);
                return true;
            }
            crossterm::event::KeyCode::Char('t') => {
                new_tab(ctx);
                return true;
            }
            crossterm::event::KeyCode::Char('w') => {
                close_tab(ctx);
                return true;
            }
            _ => {}
        }
    }
    match key.code {
        crossterm::event::KeyCode::F(8) => {
            prev_tab(ctx);
            true
        }
        crossterm::event::KeyCode::F(9) => {
            next_tab(ctx);
            true
        }
        _ => false,
    }
}
