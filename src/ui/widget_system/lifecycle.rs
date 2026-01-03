use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};

pub(crate) trait WidgetRender {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>>;
}

pub(crate) trait WidgetLifecycle: WidgetRender {
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>) -> Result<FrameLayout, Box<dyn Error>>;
    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>>;
    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
    ) -> Result<bool, Box<dyn Error>>;
}
