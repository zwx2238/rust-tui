use super::super::nav::handle_nav_key;
use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_dispatch::DispatchContext;
use crate::ui::runtime_view::{ViewAction, ViewState, handle_view_key};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::shortcuts::handle_global_shortcuts;

pub(crate) fn handle_pre_key_actions(
    ctx: &mut DispatchContext<'_>,
    view: &mut ViewState,
    key: KeyEvent,
) -> bool {
    if handle_global_shortcuts(ctx, key) {
        return true;
    }
    if handle_code_exec_reason_input(ctx, view, key) {
        return true;
    }
    if handle_stop_key(ctx, key) {
        return true;
    }
    if handle_prompt_lock_key(ctx, key) {
        return true;
    }
    handle_nav_mode_key(ctx, view, key)
}

pub(crate) fn resolve_view_action(
    ctx: &mut DispatchContext<'_>,
    view: &mut ViewState,
    key: KeyEvent,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> ViewAction {
    sync_overlay_state(ctx, view);
    handle_view_key(view, key, ctx.tabs.len(), jump_rows.len(), *ctx.active_tab)
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
            crate::ui::logic::stop_stream(&mut tab_state.app);
        }
    }
    true
}

fn handle_prompt_lock_key(ctx: &mut DispatchContext<'_>, key: KeyEvent) -> bool {
    if key.code != KeyCode::F(5) {
        return false;
    }
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
        && !crate::ui::runtime_dispatch::can_change_prompt(&tab_state.app)
    {
        crate::ui::runtime_dispatch::push_prompt_locked(tab_state);
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
