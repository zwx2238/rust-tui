use crate::ui::logic::stop_stream;
use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_events::handle_key_event;
use crate::ui::runtime_view::{ViewAction, ViewState, apply_view_action, handle_view_key};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection,
    can_change_prompt, close_all_tabs, close_other_tabs, close_tab, cycle_model, handle_nav_key,
    new_tab, next_category, next_tab, prev_category, prev_tab, push_prompt_locked,
    sync_model_selection, sync_prompt_selection,
};

pub(crate) fn handle_key_event_loop(
    key: KeyEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    if is_quit_key(key) {
        return Ok(true);
    }
    if handle_global_shortcuts(ctx, key) {
        return Ok(false);
    }
    if handle_code_exec_reason_input(ctx, view, key) {
        return Ok(false);
    }
    if handle_stop_key(ctx, key) {
        return Ok(false);
    }
    if handle_prompt_lock_key(ctx, key) {
        return Ok(false);
    }
    if handle_nav_mode_key(ctx, view, key) {
        return Ok(false);
    }
    sync_overlay_state(ctx, view);
    let action = handle_view_key(view, key, ctx.tabs.len(), jump_rows.len(), *ctx.active_tab);
    if handle_view_action_flow(ctx, layout, view, jump_rows, action, key) {
        return Ok(false);
    }
    if !view.is_chat() {
        return Ok(false);
    }
    if handle_key_event(key, ctx.tabs, *ctx.active_tab, ctx.msg_width, ctx.theme)? {
        return Ok(true);
    }
    Ok(false)
}

fn is_quit_key(key: KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q')
}

fn handle_code_exec_reason_input(
    ctx: &mut DispatchContext<'_>,
    view: &mut ViewState,
    key: KeyEvent,
) -> bool {
    if !view.overlay.is(OverlayKind::CodeExec) {
        return false;
    }
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
        && tab_state.app.code_exec_reason_target.is_some()
    {
        handle_code_exec_reason_key(&mut tab_state.app, key);
        return true;
    }
    false
}

fn handle_stop_key(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if key.code != KeyCode::F(6) {
        return false;
    }
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            crate::ui::runtime_helpers::stop_and_edit(tab_state);
        } else {
            stop_stream(&mut tab_state.app);
        }
    }
    true
}

fn handle_prompt_lock_key(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if key.code != KeyCode::F(5) {
        return false;
    }
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
        && !can_change_prompt(&tab_state.app)
    {
        push_prompt_locked(tab_state);
        return true;
    }
    false
}

fn handle_nav_mode_key(ctx: &mut DispatchContext<'_>, view: &ViewState, key: KeyEvent) -> bool {
    if !view.is_chat() {
        return false;
    }
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        return handle_nav_key(&mut tab_state.app, key);
    }
    false
}

fn sync_overlay_state(ctx: &mut DispatchContext<'_>, view: &mut ViewState) {
    if view.overlay.is(OverlayKind::CodeExec) {
        handle_code_exec_overlay_key(ctx, view);
    }
    if view.overlay.is(OverlayKind::FilePatch) {
        handle_file_patch_overlay_key(ctx, view);
    }
}

fn handle_view_action_flow(
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
    action: ViewAction,
    key: KeyEvent,
) -> bool {
    if handle_model_cycle(ctx, action) {
        return true;
    }
    if handle_fork_message(ctx, view, jump_rows, action) {
        return true;
    }
    if handle_prompt_sync(ctx, layout, view, key) {
        return true;
    }
    if handle_selection_actions(ctx, action) {
        return true;
    }
    if handle_apply_view_action(ctx, view, jump_rows, action) {
        return true;
    }
    if handle_model_sync(ctx, layout, view, key) {
        return true;
    }
    false
}

fn handle_model_cycle(ctx: &mut DispatchContext<'_>, action: ViewAction) -> bool {
    if !matches!(action, ViewAction::CycleModel) {
        return false;
    }
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        cycle_model(ctx.registry, &mut tab_state.app.model_key);
    }
    true
}

fn handle_fork_message(
    ctx: &mut DispatchContext<'_>,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
    action: ViewAction,
) -> bool {
    if let ViewAction::ForkMessage(idx) = action {
        if crate::ui::runtime_dispatch::fork_message_into_new_tab(ctx, jump_rows, idx) {
            view.overlay.close();
        }
        return true;
    }
    false
}

fn handle_prompt_sync(
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    key: KeyEvent,
) -> bool {
    if key.code == KeyCode::F(5) && view.overlay.is(OverlayKind::Prompt) {
        sync_prompt_selection(view, ctx, layout);
        return true;
    }
    false
}

fn handle_selection_actions(ctx: &mut DispatchContext<'_>, action: ViewAction) -> bool {
    if let ViewAction::SelectModel(idx) = action {
        apply_model_selection(ctx, idx);
        return true;
    }
    if let ViewAction::SelectPrompt(idx) = action {
        apply_prompt_selection(ctx, idx);
        return true;
    }
    false
}

fn handle_apply_view_action(
    ctx: &mut DispatchContext<'_>,
    view: &mut ViewState,
    jump_rows: &[crate::ui::jump::JumpRow],
    action: ViewAction,
) -> bool {
    if apply_view_action(
        action,
        jump_rows,
        ctx.tabs,
        ctx.active_tab,
        ctx.categories,
        ctx.active_category,
    ) {
        return true;
    }
    if matches!(
        action,
        ViewAction::SelectModel(_) | ViewAction::SelectPrompt(_)
    ) {
        view.overlay.close();
    }
    false
}

fn handle_model_sync(
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut ViewState,
    key: KeyEvent,
) -> bool {
    if key.code == KeyCode::F(4) && view.overlay.is(OverlayKind::Model) {
        sync_model_selection(view, ctx, layout);
        return true;
    }
    false
}

fn handle_code_exec_overlay_key(ctx: &mut DispatchContext<'_>, view: &mut ViewState) {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
        && tab_state.app.pending_code_exec.is_none()
    {
        view.overlay.close();
    }
}

fn handle_file_patch_overlay_key(ctx: &mut DispatchContext<'_>, view: &mut ViewState) {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
        && tab_state.app.pending_file_patch.is_none()
    {
        view.overlay.close();
    }
}

fn handle_code_exec_reason_key(app: &mut crate::ui::state::App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.code_exec_reason_target = None;
            app.code_exec_reason_input = tui_textarea::TextArea::default();
        }
        _ => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('u') {
                app.code_exec_reason_input = tui_textarea::TextArea::default();
                return;
            }
            let _ = app.code_exec_reason_input.input(key);
        }
    }
}

fn handle_global_shortcuts(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if handle_ctrl_shortcuts(ctx, key) {
        return true;
    }
    handle_function_tab_shortcuts(ctx, key)
}

fn handle_ctrl_shortcuts(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    if handle_ctrl_category(ctx, key) || handle_ctrl_tabs(ctx, key) {
        return true;
    }
    handle_ctrl_tab_actions(ctx, key)
}

fn handle_ctrl_category(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Up => {
            prev_category(ctx);
            true
        }
        KeyCode::Down => {
            next_category(ctx);
            true
        }
        _ => false,
    }
}

fn handle_ctrl_tabs(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Char('w') {
        close_all_tabs(ctx);
        return true;
    }
    false
}

fn handle_ctrl_tab_actions(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('o') => {
            close_other_tabs(ctx);
            true
        }
        KeyCode::Char('t') => {
            new_tab(ctx);
            true
        }
        KeyCode::Char('w') => {
            close_tab(ctx);
            true
        }
        _ => false,
    }
}

fn handle_function_tab_shortcuts(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::F(8) => {
            prev_tab(ctx);
            true
        }
        KeyCode::F(9) => {
            next_tab(ctx);
            true
        }
        _ => false,
    }
}
