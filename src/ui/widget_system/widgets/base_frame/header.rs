use crate::ui::draw::draw_header;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;

pub(super) struct HeaderWidget;

impl Widget for HeaderWidget {
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
        _ctx: &mut EventCtx<'_>,
        _event: &crossterm::event::Event,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        draw_header(
            frame.frame,
            rect,
            frame.state.theme,
            frame.state.header_note,
        );
        Ok(())
    }
}
