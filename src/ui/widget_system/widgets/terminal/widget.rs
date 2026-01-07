use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_dispatch::key_helpers::{
    handle_pre_key_actions, handle_view_action_flow, is_quit_key, resolve_view_action,
};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::bindings::bind_event;
use crate::ui::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crossterm::event::{Event, KeyCode, MouseEventKind};
use std::error::Error;

use super::layout::compute_terminal_popup_layout;
use super::render::render_terminal_popup;

pub(crate) struct TerminalWidget {
    _private: (),
}

impl TerminalWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for TerminalWidget {
    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        if !ctx.view.overlay.is(OverlayKind::Terminal) {
            return Ok(());
        }
        let popup = compute_terminal_popup_layout(layout.size);
        crate::ui::terminal::ensure_terminal_for_active_tab(
            ctx.tabs.as_mut_slice(),
            *ctx.active_tab,
            popup.terminal_area.width,
            popup.terminal_area.height,
            ctx.tx,
        );
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            Event::Key(key) => Ok(handle_key(ctx, layout, update, jump_rows, *key)),
            Event::Paste(s) => Ok(handle_paste(ctx, s)),
            Event::Mouse(m) => Ok(handle_mouse(ctx, m.kind)),
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        if !frame.view.overlay.is(OverlayKind::Terminal) {
            return Ok(());
        }
        let popup = compute_terminal_popup_layout(layout.size);
        render_terminal_popup(frame, popup.popup, popup.terminal_area);
        Ok(())
    }
}

fn handle_key(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[crate::ui::jump::JumpRow],
    key: crossterm::event::KeyEvent,
) -> EventResult {
    if is_quit_key(key) {
        return EventResult::quit();
    }
    let mut binding = bind_event(ctx, layout, update);
    if handle_pre_key_actions(&mut binding.dispatch, binding.view, key) {
        return EventResult::handled();
    }
    let action = resolve_view_action(&mut binding.dispatch, binding.view, key, jump_rows);
    if handle_view_action_flow(
        &mut binding.dispatch,
        binding.layout,
        binding.view,
        jump_rows,
        action,
        key,
    ) {
        return EventResult::handled();
    }
    if key.code == KeyCode::F(7) {
        return EventResult::handled();
    }
    send_key_to_terminal(
        binding.dispatch.tabs.as_mut_slice(),
        *binding.dispatch.active_tab,
        key,
    );
    EventResult::handled()
}

fn handle_paste(ctx: &mut EventCtx<'_>, s: &str) -> EventResult {
    let Some(tab) = ctx.tabs.get_mut(*ctx.active_tab) else {
        return EventResult::handled();
    };
    let Some(terminal) = tab.app.terminal.as_mut() else {
        return EventResult::handled();
    };
    terminal.send_paste(s);
    EventResult::handled()
}

fn handle_mouse(ctx: &mut EventCtx<'_>, kind: MouseEventKind) -> EventResult {
    let Some(tab) = ctx.tabs.get_mut(*ctx.active_tab) else {
        return EventResult::handled();
    };
    let Some(terminal) = tab.app.terminal.as_mut() else {
        return EventResult::handled();
    };
    match kind {
        MouseEventKind::ScrollUp => terminal.scroll_offset = terminal.scroll_offset.saturating_add(3),
        MouseEventKind::ScrollDown => {
            terminal.scroll_offset = terminal.scroll_offset.saturating_sub(3);
        }
        _ => {}
    }
    EventResult::handled()
}

fn send_key_to_terminal(
    tabs: &mut [crate::ui::runtime_helpers::TabState],
    active_tab: usize,
    key: crossterm::event::KeyEvent,
) {
    let Some(tab) = tabs.get_mut(active_tab) else {
        return;
    };
    let Some(terminal) = tab.app.terminal.as_mut() else {
        return;
    };
    terminal.send_key(key);
}

