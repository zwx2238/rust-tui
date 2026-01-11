use crate::framework::widget_system::widgets::jump::JumpRow;
use crate::framework::widget_system::runtime_dispatch::key_helpers::{
    handle_pre_key_actions, handle_view_action_flow, is_quit_key, resolve_view_action,
};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::bindings::bind_event;
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;

pub(super) struct GlobalKeyWidget;

impl Widget for GlobalKeyWidget {
    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Key(key) = event else {
            return Ok(EventResult::ignored());
        };
        Ok(handle_global_key(ctx, layout, update, jump_rows, *key))
    }

    fn render(
        &mut self,
        _frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

fn handle_global_key(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[crate::framework::widget_system::widgets::jump::JumpRow],
    key: crossterm::event::KeyEvent,
) -> EventResult {
    if key_debug_enabled() && let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab) {
        crate::framework::widget_system::notice::push_notice(&mut tab_state.app, format_key_event(key));
    }
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
    EventResult::ignored()
}

fn key_debug_enabled() -> bool {
    std::env::var_os("DEEPCHAT_KEY_DEBUG").is_some()
}

fn format_key_event(key: crossterm::event::KeyEvent) -> String {
    let mods = format_modifiers(key.modifiers);
    format!("KeyEvent: code={:?} mods={mods}", key.code)
}

fn format_modifiers(mods: crossterm::event::KeyModifiers) -> String {
    if mods.is_empty() {
        return "NONE".to_string();
    }
    let mut parts = Vec::new();
    if mods.contains(crossterm::event::KeyModifiers::CONTROL) {
        parts.push("CTRL");
    }
    if mods.contains(crossterm::event::KeyModifiers::SHIFT) {
        parts.push("SHIFT");
    }
    if mods.contains(crossterm::event::KeyModifiers::ALT) {
        parts.push("ALT");
    }
    parts.join("+")
}
