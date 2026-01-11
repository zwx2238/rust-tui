mod box_constraints;
pub(crate) mod commands;
pub(crate) mod draw;
mod events;
pub(crate) mod interaction;
pub(crate) mod layout;
pub(crate) mod notice;
pub(crate) mod overlay;
pub(crate) mod runtime;
pub(crate) mod runtime_dispatch;
pub(crate) mod runtime_loop_helpers;
pub(crate) mod runtime_tick;
mod context;
mod lifecycle;
mod render;
mod widget_pod;
pub(crate) mod widgets;

use std::error::Error;

pub(crate) use box_constraints::BoxConstraints;
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
    ) -> Result<crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout, Box<dyn Error>> {
        let size = ctx.terminal.size()?;
        let size = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let bc = BoxConstraints::tight(ratatui::layout::Size {
            width: size.width,
            height: size.height,
        });
        let _ = self.root.measure(ctx, bc)?;
        let mut layout = crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout {
            size,
            layout: crate::framework::widget_system::layout::empty_layout_info(),
        };
        self.root.place(ctx, &mut layout, size)?;
        Ok(layout)
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        let update = self.frame.update(ctx, layout)?;
        self.root.update(ctx, layout, &update)?;
        Ok(update)
    }

    pub(crate) fn render<'a>(
        &mut self,
        ctx: &'a mut RenderCtx<'a>,
        layout: &'a crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout,
        update: &'a UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        render_root(ctx, layout, update, &mut self.root)
    }

    pub(crate) fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
        event: &crossterm::event::Event,
    ) -> Result<bool, Box<dyn Error>> {
        let result = self
            .root
            .event(ctx, event, layout, update, layout.size)?;
        Ok(result.quit)
    }

    // render_view is no longer supported with the new render pipeline.
}
mod bindings;
