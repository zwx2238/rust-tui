use crate::ui::runtime_dispatch::key_helpers::{
    handle_pre_key_actions, handle_view_action_flow, is_quit_key, resolve_view_action,
};
use crate::ui::widget_system::bindings::bind_event;
use crate::ui::widget_system::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

pub(super) struct GlobalKeyWidget;

impl Widget for GlobalKeyWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

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
    EventResult::ignored()
}
