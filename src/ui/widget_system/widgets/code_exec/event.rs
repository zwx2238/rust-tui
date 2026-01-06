use crate::ui::code_exec_popup_layout::{CodeExecPopupLayout, code_exec_popup_layout};
use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::bindings::bind_event;
use crate::ui::widget_system::context::{EventCtx, UpdateOutput};
use crate::ui::widget_system::lifecycle::EventResult;
use crate::ui::widget_system::widgets::overlay_table::OverlayTableController;
use std::error::Error;

use super::buttons::{CodeExecButtonParams, handle_code_exec_buttons};
use super::helpers::{
    is_ctrl_c, is_mouse_down, is_mouse_drag, is_mouse_moved, is_mouse_up, point_in_rect,
    scroll_delta,
};
use super::render::hover_at;
use super::scroll::handle_code_exec_scroll;
use super::selection::{
    clear_code_exec_selection, copy_code_exec_selection, handle_code_exec_selection_drag,
    handle_code_exec_selection_start,
};
use super::widget::CodeExecWidget;

pub(super) fn handle_mouse_event(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    m: crossterm::event::MouseEvent,
) -> Result<EventResult, Box<dyn Error>> {
    let Some(state) = mouse_state(ctx, layout) else {
        return Ok(EventResult::ignored());
    };
    Ok(handle_mouse_with_state(
        widget, ctx, layout, update, m, state,
    ))
}

pub(super) fn handle_key_event(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[JumpRow],
    event: &crossterm::event::Event,
) -> Result<EventResult, Box<dyn Error>> {
    if let crossterm::event::Event::Key(key) = event
        && is_ctrl_c(*key)
        && try_copy_selection(ctx, layout)
    {
        return Ok(EventResult::ignored());
    }
    let binding = bind_event(ctx, layout, update);
    let mut controller = OverlayTableController {
        dispatch: binding.dispatch,
        layout: binding.layout,
        view: binding.view,
        jump_rows,
    };
    controller.handle_event(event)
}

fn try_copy_selection(ctx: &mut EventCtx<'_>, layout: &FrameLayout) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
        && let Some(pending) = tab_state.app.pending_code_exec.clone()
    {
        return copy_code_exec_selection(tab_state, &pending, layout, ctx.theme);
    }
    false
}

fn mouse_state(ctx: &mut EventCtx<'_>, layout: &FrameLayout) -> Option<MouseState> {
    let active_tab = *ctx.active_tab;
    let pending = ctx
        .tabs
        .get(active_tab)
        .and_then(|tab| tab.app.pending_code_exec.clone())?;
    let reason_target = ctx
        .tabs
        .get(active_tab)
        .map(|tab| tab.app.code_exec_reason_target)
        .unwrap_or(None);
    let popup = code_exec_popup_layout(layout.size, reason_target.is_some());
    Some(MouseState {
        active_tab,
        pending,
        popup,
        reason_target,
    })
}

fn handle_mouse_with_state(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    m: crossterm::event::MouseEvent,
    state: MouseState,
) -> EventResult {
    if let Some(result) = try_drag(ctx, m, &state) {
        return result;
    }
    if let Some(result) = try_mouse_up(ctx, &state, m) {
        return result;
    }
    if let Some(result) = try_mouse_move(ctx, &state, m) {
        return result;
    }
    if let Some(result) = try_scroll(ctx, &state, m) {
        return result;
    }
    if is_mouse_down(m.kind) {
        return handle_mouse_down(widget, ctx, layout, update, m, &state);
    }
    EventResult::ignored()
}

fn try_drag(
    ctx: &mut EventCtx<'_>,
    m: crossterm::event::MouseEvent,
    state: &MouseState,
) -> Option<EventResult> {
    if !is_mouse_drag(m.kind) {
        return None;
    }
    if handle_drag(ctx, state, m) {
        return Some(EventResult::handled());
    }
    None
}

fn try_mouse_up(
    ctx: &mut EventCtx<'_>,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
) -> Option<EventResult> {
    if !is_mouse_up(m.kind) {
        return None;
    }
    if handle_mouse_up_action(ctx, state) {
        return Some(EventResult::handled());
    }
    None
}

fn try_mouse_move(
    ctx: &mut EventCtx<'_>,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
) -> Option<EventResult> {
    if !is_mouse_moved(m.kind) {
        return None;
    }
    handle_mouse_move(ctx, state, m);
    Some(EventResult::handled())
}

fn try_scroll(
    ctx: &mut EventCtx<'_>,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
) -> Option<EventResult> {
    let delta = scroll_delta(m.kind)?;
    if handle_scroll(ctx, state, m, delta) {
        return Some(EventResult::handled());
    }
    None
}

fn handle_drag(
    ctx: &mut EventCtx<'_>,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        return handle_code_exec_selection_drag(
            tab_state,
            &state.pending,
            state.popup,
            ctx.theme,
            m,
        );
    }
    false
}

fn handle_mouse_up_action(ctx: &mut EventCtx<'_>, state: &MouseState) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        return clear_code_exec_selection(tab_state);
    }
    false
}

fn handle_mouse_move(ctx: &mut EventCtx<'_>, state: &MouseState, m: crossterm::event::MouseEvent) {
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        tab_state.app.code_exec_hover = hover_at(m, state.popup, state.reason_target.is_some());
    }
}

fn handle_scroll(
    ctx: &mut EventCtx<'_>,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
    delta: i32,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        return handle_code_exec_scroll(
            m,
            ctx.theme,
            tab_state,
            &state.pending,
            state.popup,
            delta,
        );
    }
    false
}

fn handle_mouse_down(
    widget: &mut CodeExecWidget,
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    m: crossterm::event::MouseEvent,
    state: &MouseState,
) -> EventResult {
    if try_selection_start(ctx, state, m) {
        return EventResult::handled();
    }
    let params = CodeExecButtonParams {
        active_tab: state.active_tab,
        layout: state.popup,
        theme: ctx.theme,
        frame_layout: layout,
        update,
    };
    if handle_code_exec_buttons(widget, ctx, m, &params) {
        return EventResult::handled();
    }
    if handle_click_outside(ctx, layout, state, m) {
        return EventResult::handled();
    }
    EventResult::ignored()
}

fn try_selection_start(
    ctx: &mut EventCtx<'_>,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        return handle_code_exec_selection_start(
            tab_state,
            &state.pending,
            state.popup,
            ctx.theme,
            m,
        );
    }
    false
}

fn handle_click_outside(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
) -> bool {
    if !click_outside_tabs(layout, m) {
        return false;
    }
    ctx.view.overlay.close();
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        tab_state.app.code_exec_hover = None;
    }
    true
}

fn click_outside_tabs(layout: &FrameLayout, m: crossterm::event::MouseEvent) -> bool {
    !point_in_rect(m.column, m.row, layout.layout.tabs_area)
        && !point_in_rect(m.column, m.row, layout.layout.category_area)
}

struct MouseState {
    active_tab: usize,
    pending: crate::ui::state::PendingCodeExec,
    popup: CodeExecPopupLayout,
    reason_target: Option<crate::ui::state::CodeExecReasonTarget>,
}
