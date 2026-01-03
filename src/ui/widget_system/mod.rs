mod context;
mod events;
mod lifecycle;
mod render;
mod widgets;
#[cfg(test)]
mod widget_system_tests;

use std::error::Error;

pub(crate) use context::{
    EventCtx, LayoutCtx, RenderCtx, UpdateCtx, UpdateOutput,
};
pub(crate) use lifecycle::Widget;
use render::render_root;
#[cfg(test)]
use render::render_root_view;
use widgets::RootWidget;

pub(crate) struct WidgetSystem {
    root: RootWidget,
}

impl WidgetSystem {
    pub(crate) fn new() -> Self {
        Self {
            root: RootWidget::new(),
        }
    }

    pub(crate) fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
    ) -> Result<crate::ui::runtime_loop_steps::FrameLayout, Box<dyn Error>> {
        self.root.layout(ctx)
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        self.root.update(ctx, layout)
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
    ) -> Result<bool, Box<dyn Error>> {
        self.root.event(ctx, layout, update, jump_rows)
    }

    #[cfg(test)]
    pub(crate) fn render_view<'a>(
        &mut self,
        ctx: &mut crate::ui::render_context::RenderContext<'a>,
        view: &mut crate::ui::runtime_view::ViewState,
    ) -> Result<Vec<crate::ui::jump::JumpRow>, Box<dyn Error>> {
        render_root_view(ctx, view, &mut self.root)
    }
}
mod bindings;
