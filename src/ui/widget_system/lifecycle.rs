use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::box_constraints::BoxConstraints;
use crossterm::event::Event;
use ratatui::layout::{Rect, Size};
use std::error::Error;

use super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct EventResult {
    pub(crate) handled: bool,
    pub(crate) quit: bool,
}

impl EventResult {
    pub(crate) fn ignored() -> Self {
        Self {
            handled: false,
            quit: false,
        }
    }

    pub(crate) fn handled() -> Self {
        Self {
            handled: true,
            quit: false,
        }
    }

    pub(crate) fn quit() -> Self {
        Self {
            handled: true,
            quit: true,
        }
    }
}

pub(crate) trait Widget {
    fn measure(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        Ok(bc.constrain(bc.max))
    }

    fn place(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &mut FrameLayout,
        _rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>>;
    fn event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _event: &Event,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _jump_rows: &[JumpRow],
        _rect: Rect,
    ) -> Result<EventResult, Box<dyn Error>>;
    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: Rect,
    ) -> Result<(), Box<dyn Error>>;
}

impl Widget for Box<dyn Widget> {
    fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        (**self).measure(ctx, bc)
    }

    fn place(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &mut FrameLayout,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        (**self).place(ctx, layout, rect)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        (**self).update(ctx, layout, update)
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        rect: Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        (**self).event(ctx, event, layout, update, jump_rows, rect)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        (**self).render(frame, layout, update, rect)
    }
}
