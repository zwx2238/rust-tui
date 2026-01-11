use super::popup_layout::{FilePatchPopupLayout, file_patch_popup_layout};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::bindings::bind_event;
use crate::framework::widget_system::context::{EventCtx, UpdateOutput};
use crate::framework::widget_system::lifecycle::EventResult;
use crate::framework::widget_system::widgets::overlay_table::OverlayTableController;
use std::error::Error;

use super::buttons::{FilePatchButtonParams, handle_file_patch_buttons};
use super::helpers::{
    is_ctrl_c, is_mouse_down, is_mouse_drag, is_mouse_moved, is_mouse_up, point_in_rect,
    scroll_delta,
};
use super::scroll::handle_file_patch_scroll;
use super::selection::{
    clear_file_patch_selection, copy_file_patch_selection, handle_file_patch_selection_drag,
    handle_file_patch_selection_start, hover_at,
};
use super::widget::FilePatchWidget;

pub(super) fn handle_mouse_event(
    widget: &mut FilePatchWidget,
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
    };
    controller.handle_event(event)
}

fn try_copy_selection(ctx: &mut EventCtx<'_>, layout: &FrameLayout) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
        && let Some(pending) = tab_state.app.pending_file_patch.clone()
    {
        return copy_file_patch_selection(tab_state, &pending, layout, ctx.theme);
    }
    false
}

fn mouse_state(ctx: &mut EventCtx<'_>, layout: &FrameLayout) -> Option<MouseState> {
    let active_tab = *ctx.active_tab;
    let pending = ctx
        .tabs
        .get(active_tab)
        .and_then(|tab| tab.app.pending_file_patch.clone())?;
    let popup = file_patch_popup_layout(layout.size);
    Some(MouseState {
        active_tab,
        pending,
        popup,
    })
}

fn handle_mouse_with_state(
    widget: &mut FilePatchWidget,
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
    if let Some(result) = try_scroll(ctx, m, &state) {
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
    m: crossterm::event::MouseEvent,
    state: &MouseState,
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
        return handle_file_patch_selection_drag(
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
        return clear_file_patch_selection(tab_state);
    }
    false
}

fn handle_mouse_move(ctx: &mut EventCtx<'_>, state: &MouseState, m: crossterm::event::MouseEvent) {
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        tab_state.app.file_patch_hover = hover_at(m, state.popup);
    }
}

fn handle_scroll(
    ctx: &mut EventCtx<'_>,
    state: &MouseState,
    m: crossterm::event::MouseEvent,
    delta: i32,
) -> bool {
    if let Some(tab_state) = ctx.tabs.get_mut(state.active_tab) {
        return handle_file_patch_scroll(
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
    widget: &mut FilePatchWidget,
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    m: crossterm::event::MouseEvent,
    state: &MouseState,
) -> EventResult {
    if try_selection_start(ctx, state, m) {
        return EventResult::handled();
    }
    let params = FilePatchButtonParams {
        active_tab: state.active_tab,
        popup: state.popup,
        theme: ctx.theme,
        layout,
        update,
    };
    if handle_file_patch_buttons(widget, ctx, m, &params) {
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
        return handle_file_patch_selection_start(
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
        tab_state.app.file_patch_hover = None;
    }
    true
}

fn click_outside_tabs(layout: &FrameLayout, m: crossterm::event::MouseEvent) -> bool {
    !point_in_rect(m.column, m.row, layout.layout.tabs_area)
        && !point_in_rect(m.column, m.row, layout.layout.category_area)
}

struct MouseState {
    active_tab: usize,
    pending: crate::framework::widget_system::runtime::state::PendingFilePatch,
    popup: FilePatchPopupLayout,
}
