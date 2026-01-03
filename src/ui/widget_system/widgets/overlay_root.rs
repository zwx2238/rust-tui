use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_view::ViewState;
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::widget_system::widget_pod::WidgetPod;
use std::error::Error;

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::ui::runtime_loop_steps::FrameLayout;

use super::code_exec::CodeExecWidget;
use super::file_patch::FilePatchWidget;
use super::help::HelpWidget;
use super::jump::JumpWidget;
use super::model::ModelWidget;
use super::prompt::PromptWidget;
use super::summary::SummaryWidget;

pub(crate) struct OverlayRootWidget {
    summary: WidgetPod<SummaryWidget>,
    jump: WidgetPod<JumpWidget>,
    model: WidgetPod<ModelWidget>,
    prompt: WidgetPod<PromptWidget>,
    code_exec: WidgetPod<CodeExecWidget>,
    file_patch: WidgetPod<FilePatchWidget>,
    help: WidgetPod<HelpWidget>,
}

impl OverlayRootWidget {
    pub(crate) fn new() -> Self {
        Self {
            summary: WidgetPod::new(SummaryWidget::new()),
            jump: WidgetPod::new(JumpWidget::new()),
            model: WidgetPod::new(ModelWidget::new()),
            prompt: WidgetPod::new(PromptWidget::new()),
            code_exec: WidgetPod::new(CodeExecWidget::new()),
            file_patch: WidgetPod::new(FilePatchWidget::new()),
            help: WidgetPod::new(HelpWidget::new()),
        }
    }

    fn active_kind(view: &ViewState) -> Option<OverlayKind> {
        view.overlay.active
    }
}

impl Widget for OverlayRootWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.summary.set_rect(rect);
        self.jump.set_rect(rect);
        self.model.set_rect(rect);
        self.prompt.set_rect(rect);
        self.code_exec.set_rect(rect);
        self.file_patch.set_rect(rect);
        self.help.set_rect(rect);
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        match Self::active_kind(ctx.view) {
            Some(OverlayKind::Summary) => self.summary.update(ctx, layout, update)?,
            Some(OverlayKind::Jump) => self.jump.update(ctx, layout, update)?,
            Some(OverlayKind::Model) => self.model.update(ctx, layout, update)?,
            Some(OverlayKind::Prompt) => self.prompt.update(ctx, layout, update)?,
            Some(OverlayKind::CodeExec) => self.code_exec.update(ctx, layout, update)?,
            Some(OverlayKind::FilePatch) => self.file_patch.update(ctx, layout, update)?,
            Some(OverlayKind::Help) => self.help.update(ctx, layout, update)?,
            None => {}
        }
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let result = match Self::active_kind(ctx.view) {
            Some(OverlayKind::Summary) => self.summary.event(ctx, event, layout, update, jump_rows),
            Some(OverlayKind::Jump) => self.jump.event(ctx, event, layout, update, jump_rows),
            Some(OverlayKind::Model) => self.model.event(ctx, event, layout, update, jump_rows),
            Some(OverlayKind::Prompt) => self.prompt.event(ctx, event, layout, update, jump_rows),
            Some(OverlayKind::CodeExec) => self.code_exec.event(ctx, event, layout, update, jump_rows),
            Some(OverlayKind::FilePatch) => self.file_patch.event(ctx, event, layout, update, jump_rows),
            Some(OverlayKind::Help) => self.help.event(ctx, event, layout, update, jump_rows),
            None => return Ok(EventResult::ignored()),
        }?;
        return Ok(result);
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        match Self::active_kind(frame.view) {
            Some(OverlayKind::Summary) => self.summary.render(frame, layout, update)?,
            Some(OverlayKind::Jump) => self.jump.render(frame, layout, update)?,
            Some(OverlayKind::Model) => self.model.render(frame, layout, update)?,
            Some(OverlayKind::Prompt) => self.prompt.render(frame, layout, update)?,
            Some(OverlayKind::CodeExec) => self.code_exec.render(frame, layout, update)?,
            Some(OverlayKind::FilePatch) => self.file_patch.render(frame, layout, update)?,
            Some(OverlayKind::Help) => self.help.render(frame, layout, update)?,
            None => {}
        }
        Ok(())
    }
}
