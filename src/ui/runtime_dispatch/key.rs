use crate::ui::runtime_view::ViewState;
use crossterm::event::KeyEvent;

use super::key_helpers::{
    handle_chat_input, handle_pre_key_actions, handle_view_action_flow, is_quit_key,
    resolve_view_action,
};

use super::{DispatchContext, LayoutContext};

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
    if handle_pre_key_actions(ctx, view, key) {
        return Ok(false);
    }
    let action = resolve_view_action(ctx, view, key, jump_rows);
    if handle_view_action_flow(ctx, layout, view, jump_rows, action, key) {
        return Ok(false);
    }
    if !view.is_chat() {
        return Ok(false);
    }
    handle_chat_input(ctx, key)
}
