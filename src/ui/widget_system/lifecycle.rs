use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};

pub(crate) trait Widget {
    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>) -> Result<FrameLayout, Box<dyn Error>>;
    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>>;
    fn event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _jump_rows: &[JumpRow],
    ) -> Result<bool, Box<dyn Error>>;
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>>;
}
