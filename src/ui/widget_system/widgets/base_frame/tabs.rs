use crate::ui::widget_system::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::draw::draw_tabs;
use std::error::Error;

use super::helpers::handle_tab_category_mouse_down;

pub(super) struct TabsWidget;

impl Widget for TabsWidget {
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
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        handle_tab_category_mouse_down(ctx, layout, update, rect, *m)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        draw_tabs(
            frame.frame,
            rect,
            frame.state.tab_labels,
            frame.state.active_tab_pos,
            frame.state.theme,
            frame.state.startup_text,
        );
        Ok(())
    }
}
