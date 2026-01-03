use crate::ui::jump::JumpRow;
use std::error::Error;

use super::common::{event_base, layout_base, update_base};
use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::Widget;

pub(crate) struct CodeExecWidget;

impl Widget for CodeExecWidget {
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>) -> Result<crate::ui::runtime_loop_steps::FrameLayout, Box<dyn Error>> {
        layout_base(ctx)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        update_base(ctx, layout)
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
    ) -> Result<bool, Box<dyn Error>> {
        event_base(ctx, layout, update, jump_rows)
    }

    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        crate::ui::overlay_render_tool::render_code_exec_overlay(frame.ctx)
    }
}
