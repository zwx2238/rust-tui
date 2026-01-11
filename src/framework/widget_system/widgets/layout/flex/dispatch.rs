use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::context::{EventCtx, UpdateOutput};
use crate::framework::widget_system::lifecycle::EventResult;
use crate::framework::widget_system::widget_pod::WidgetPod;
use ratatui::layout::Rect;
use std::error::Error;

pub(super) struct MouseDispatch<'a, 'ctx> {
    pub(super) ctx: &'a mut EventCtx<'ctx>,
    pub(super) event: &'a crossterm::event::Event,
    pub(super) layout: &'a FrameLayout,
    pub(super) update: &'a UpdateOutput,
    pub(super) jump_rows: &'a [crate::framework::widget_system::widgets::jump::JumpRow],
    pub(super) rect: Rect,
}

pub(super) fn dispatch_mouse_pair<
    A: crate::framework::widget_system::lifecycle::Widget,
    B: crate::framework::widget_system::lifecycle::Widget,
>(
    a: &mut WidgetPod<A>,
    b: &mut WidgetPod<B>,
    params: MouseDispatch<'_, '_>,
) -> Result<EventResult, Box<dyn Error>> {
    let Some((x, y)) = mouse_pos(params.event) else {
        return Ok(EventResult::ignored());
    };
    if !point_in_rect(x, y, params.rect) {
        return Ok(EventResult::ignored());
    }
    if b.contains(x, y) {
        return b.event(
            params.ctx,
            params.event,
            params.layout,
            params.update,
            params.jump_rows,
        );
    }
    if a.contains(x, y) {
        return a.event(
            params.ctx,
            params.event,
            params.layout,
            params.update,
            params.jump_rows,
        );
    }
    Ok(EventResult::ignored())
}

pub(super) fn dispatch_mouse_triple<
    A: crate::framework::widget_system::lifecycle::Widget,
    B: crate::framework::widget_system::lifecycle::Widget,
    C: crate::framework::widget_system::lifecycle::Widget,
>(
    a: &mut WidgetPod<A>,
    b: &mut WidgetPod<B>,
    c: &mut WidgetPod<C>,
    params: MouseDispatch<'_, '_>,
) -> Result<EventResult, Box<dyn Error>> {
    let Some((x, y)) = mouse_pos(params.event) else {
        return Ok(EventResult::ignored());
    };
    if !point_in_rect(x, y, params.rect) {
        return Ok(EventResult::ignored());
    }
    if c.contains(x, y) {
        return c.event(
            params.ctx,
            params.event,
            params.layout,
            params.update,
            params.jump_rows,
        );
    }
    if b.contains(x, y) {
        return b.event(
            params.ctx,
            params.event,
            params.layout,
            params.update,
            params.jump_rows,
        );
    }
    if a.contains(x, y) {
        return a.event(
            params.ctx,
            params.event,
            params.layout,
            params.update,
            params.jump_rows,
        );
    }
    Ok(EventResult::ignored())
}

fn mouse_pos(event: &crossterm::event::Event) -> Option<(u16, u16)> {
    let crossterm::event::Event::Mouse(m) = event else {
        return None;
    };
    Some((m.column, m.row))
}

fn point_in_rect(x: u16, y: u16, rect: Rect) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
