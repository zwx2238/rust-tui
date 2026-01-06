mod context;
mod lifecycle;
mod render;
mod widget_pod;
#[cfg(test)]
mod widget_system_tests;
mod widgets;

use std::error::Error;

pub(crate) use context::{EventCtx, LayoutCtx, RenderCtx, UpdateCtx, UpdateOutput};
pub(crate) use lifecycle::{EventResult, Widget};
use render::render_root;
use widgets::{FrameLifecycle, RootWidget};

pub(crate) struct WidgetSystem {
    frame: FrameLifecycle,
    root: RootWidget,
}

impl WidgetSystem {
    pub(crate) fn new() -> Self {
        Self {
            frame: FrameLifecycle,
            root: RootWidget::new(),
        }
    }

    pub(crate) fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
    ) -> Result<crate::ui::runtime_loop_steps::FrameLayout, Box<dyn Error>> {
        let layout = self.frame.layout(ctx)?;
        self.root.layout(ctx, &layout, layout.size)?;
        Ok(layout)
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        let update = self.frame.update(ctx, layout)?;
        self.root.update(ctx, layout, &update)?;
        Ok(update)
    }

    pub(crate) fn render<'a>(
        &mut self,
        ctx: &'a mut RenderCtx<'a>,
        layout: &'a crate::ui::runtime_loop_steps::FrameLayout,
        update: &'a UpdateOutput,
    ) -> Result<Vec<crate::ui::jump::JumpRow>, Box<dyn Error>> {
        render_root(ctx, layout, update, &mut self.root)
    }

    pub(crate) fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        event: &crossterm::event::Event,
    ) -> Result<bool, Box<dyn Error>> {
        let result = self
            .root
            .event(ctx, event, layout, update, jump_rows, layout.size)?;
        Ok(result.quit)
    }

    // render_view is no longer supported with the new render pipeline.
}
mod bindings;
