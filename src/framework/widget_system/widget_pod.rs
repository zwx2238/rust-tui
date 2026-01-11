use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::box_constraints::BoxConstraints;
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::layout::Size;
use std::error::Error;

use super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};

pub(crate) struct WidgetPod<W: Widget> {
    widget: W,
    rect: Rect,
}

impl<W: Widget> WidgetPod<W> {
    pub(crate) fn new(widget: W) -> Self {
        Self {
            widget,
            rect: Rect::new(0, 0, 0, 0),
        }
    }

    pub(crate) fn widget(&self) -> &W {
        &self.widget
    }

    pub(crate) fn rect(&self) -> Rect {
        self.rect
    }

    pub(crate) fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        self.widget.measure(ctx, bc)
    }

    pub(crate) fn place(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &mut FrameLayout,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.rect = rect;
        self.widget.place(ctx, layout, rect)
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.widget.update(ctx, layout, update)
    }

    pub(crate) fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::framework::widget_system::widgets::jump::JumpRow],
    ) -> Result<EventResult, Box<dyn Error>> {
        self.widget
            .event(ctx, event, layout, update, jump_rows, self.rect)
    }

    pub(crate) fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.widget.render(frame, layout, update, self.rect)
    }

    pub(crate) fn contains(&self, column: u16, row: u16) -> bool {
        column >= self.rect.x
            && column < self.rect.x + self.rect.width
            && row >= self.rect.y
            && row < self.rect.y + self.rect.height
    }
}
