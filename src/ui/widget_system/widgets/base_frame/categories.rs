use crate::ui::draw::draw_categories;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::BoxConstraints;
use crate::ui::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use ratatui::layout::Size;
use std::error::Error;

use super::helpers::handle_tab_category_mouse_down;

pub(super) struct CategoriesWidget;

impl Widget for CategoriesWidget {
    fn measure(&mut self, ctx: &mut LayoutCtx<'_>, bc: BoxConstraints) -> Result<Size, Box<dyn Error>> {
        let width = crate::ui::runtime_layout::compute_sidebar_width(ctx.categories, bc.max.width);
        Ok(Size {
            width,
            height: bc.max.height,
        })
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
        draw_categories(
            frame.frame,
            rect,
            frame.state.categories,
            frame.state.active_category,
            frame.state.theme,
        );
        Ok(())
    }
}
