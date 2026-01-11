use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;

use super::event::{handle_key_event, handle_mouse_event};
use super::render::render_code_exec_overlay;

pub(crate) struct CodeExecWidget {
    pub(super) approve_btn: crate::framework::widget_system::widgets::button::ButtonWidget,
    pub(super) deny_btn: crate::framework::widget_system::widgets::button::ButtonWidget,
    pub(super) stop_btn: crate::framework::widget_system::widgets::button::ButtonWidget,
    pub(super) exit_btn: crate::framework::widget_system::widgets::button::ButtonWidget,
}

impl CodeExecWidget {
    pub(crate) fn new() -> Self {
        Self {
            approve_btn: crate::framework::widget_system::widgets::button::ButtonWidget::new("确认执行"),
            deny_btn: crate::framework::widget_system::widgets::button::ButtonWidget::new("取消拒绝"),
            stop_btn: crate::framework::widget_system::widgets::button::ButtonWidget::new("停止执行"),
            exit_btn: crate::framework::widget_system::widgets::button::ButtonWidget::new("退出"),
        }
    }
}

impl Widget for CodeExecWidget {
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
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            crossterm::event::Event::Mouse(m) => handle_mouse_event(self, ctx, layout, update, *m),
            crossterm::event::Event::Key(_) => {
                handle_key_event(ctx, layout, update, event)
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        render_code_exec_overlay(self, frame, layout, update, rect)
    }
}
