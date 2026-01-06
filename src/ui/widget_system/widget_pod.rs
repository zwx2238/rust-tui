use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crossterm::event::Event;
use ratatui::layout::Rect;
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

    pub(crate) fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub(crate) fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &FrameLayout,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.rect = rect;
        self.widget.layout(ctx, layout, rect)
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
        jump_rows: &[crate::ui::jump::JumpRow],
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
