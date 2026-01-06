use std::error::Error;

use crate::ui::widget_system::widget_pod::WidgetPod;
use crate::ui::widget_system::{EventResult, Widget};

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::base_frame::{BaseFrameWidget, NoticeWidget};
use super::overlay_root::OverlayRootWidget;

pub(crate) struct RootWidget {
    base: WidgetPod<BaseFrameWidget>,
    overlay: WidgetPod<OverlayRootWidget>,
    notice: WidgetPod<NoticeWidget>,
}

impl RootWidget {
    pub(crate) fn new() -> Self {
        Self {
            base: WidgetPod::new(BaseFrameWidget::new()),
            overlay: WidgetPod::new(OverlayRootWidget::new()),
            notice: WidgetPod::new(NoticeWidget),
        }
    }
}

impl Widget for RootWidget {
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.base.layout(ctx, layout, rect)?;
        self.overlay.layout(ctx, layout, rect)?;
        self.notice.layout(ctx, layout, rect)?;
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.base.update(ctx, layout, update)?;
        self.overlay.update(ctx, layout, update)?;
        self.notice.update(ctx, layout, update)?;
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        if ctx.view.overlay.active.is_some() {
            return self.overlay.event(ctx, event, layout, update, jump_rows);
        }
        self.base.event(ctx, event, layout, update, jump_rows)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        frame.jump_rows.clear();
        self.base.render(frame, layout, update)?;
        self.overlay.render(frame, layout, update)?;
        self.notice.render(frame, layout, update)?;
        Ok(())
    }
}
