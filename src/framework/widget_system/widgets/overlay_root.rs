use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_view::ViewState;
use crate::framework::widget_system::BoxConstraints;
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use crate::framework::widget_system::widget_pod::WidgetPod;
use ratatui::layout::Size;
use std::error::Error;

use crate::ui::runtime_events::handle_tab_category_click;
use crate::framework::widget_system::bindings::bind_event;

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::ui::runtime_loop_steps::FrameLayout;

use super::code_exec::CodeExecWidget;
use super::file_patch::FilePatchWidget;
use super::help::HelpWidget;
use super::jump::JumpWidget;
use super::model::ModelWidget;
use super::prompt::PromptWidget;
use super::question_review::QuestionReviewWidget;
use super::summary::SummaryWidget;
use super::terminal::TerminalWidget;

pub(crate) struct OverlayRootWidget {
    summary: WidgetPod<SummaryWidget>,
    jump: WidgetPod<JumpWidget>,
    model: WidgetPod<ModelWidget>,
    prompt: WidgetPod<PromptWidget>,
    question_review: WidgetPod<QuestionReviewWidget>,
    code_exec: WidgetPod<CodeExecWidget>,
    file_patch: WidgetPod<FilePatchWidget>,
    terminal: WidgetPod<TerminalWidget>,
    help: WidgetPod<HelpWidget>,
}

impl OverlayRootWidget {
    pub(crate) fn new() -> Self {
        Self {
            summary: WidgetPod::new(SummaryWidget::new()),
            jump: WidgetPod::new(JumpWidget::new()),
            model: WidgetPod::new(ModelWidget::new()),
            prompt: WidgetPod::new(PromptWidget::new()),
            question_review: WidgetPod::new(QuestionReviewWidget::new()),
            code_exec: WidgetPod::new(CodeExecWidget::new()),
            file_patch: WidgetPod::new(FilePatchWidget::new()),
            terminal: WidgetPod::new(TerminalWidget::new()),
            help: WidgetPod::new(HelpWidget::new()),
        }
    }

    fn active_kind(view: &ViewState) -> Option<OverlayKind> {
        view.overlay.active
    }
}

impl Widget for OverlayRootWidget {
    fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        let _ = self.summary.measure(ctx, bc)?;
        let _ = self.jump.measure(ctx, bc)?;
        let _ = self.model.measure(ctx, bc)?;
        let _ = self.prompt.measure(ctx, bc)?;
        let _ = self.question_review.measure(ctx, bc)?;
        let _ = self.code_exec.measure(ctx, bc)?;
        let _ = self.file_patch.measure(ctx, bc)?;
        let _ = self.terminal.measure(ctx, bc)?;
        let _ = self.help.measure(ctx, bc)?;
        Ok(bc.max)
    }

    fn place(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &mut FrameLayout,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.summary.place(ctx, layout, rect)?;
        self.jump.place(ctx, layout, rect)?;
        self.model.place(ctx, layout, rect)?;
        self.prompt.place(ctx, layout, rect)?;
        self.question_review.place(ctx, layout, rect)?;
        self.code_exec.place(ctx, layout, rect)?;
        self.file_patch.place(ctx, layout, rect)?;
        self.terminal.place(ctx, layout, rect)?;
        self.help.place(ctx, layout, rect)?;
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
            Some(OverlayKind::QuestionReview) => self.question_review.update(ctx, layout, update)?,
            Some(OverlayKind::CodeExec) => self.code_exec.update(ctx, layout, update)?,
            Some(OverlayKind::FilePatch) => self.file_patch.update(ctx, layout, update)?,
            Some(OverlayKind::Terminal) => self.terminal.update(ctx, layout, update)?,
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
        if try_tab_category_click(ctx, event, layout, update)? {
            return Ok(EventResult::handled());
        }
        dispatch_overlay_event(self, ctx, event, layout, update, jump_rows)
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
            Some(OverlayKind::QuestionReview) => self.question_review.render(frame, layout, update)?,
            Some(OverlayKind::CodeExec) => self.code_exec.render(frame, layout, update)?,
            Some(OverlayKind::FilePatch) => self.file_patch.render(frame, layout, update)?,
            Some(OverlayKind::Terminal) => self.terminal.render(frame, layout, update)?,
            Some(OverlayKind::Help) => self.help.render(frame, layout, update)?,
            None => {}
        }
        Ok(())
    }
}

fn try_tab_category_click(
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    layout: &FrameLayout,
    update: &UpdateOutput,
) -> Result<bool, Box<dyn Error>> {
    let crossterm::event::Event::Mouse(m) = event else {
        return Ok(false);
    };
    if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
        return Ok(false);
    }
    let binding = bind_event(ctx, layout, update);
    let handled = handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
        mouse_x: m.column,
        mouse_y: m.row,
        tabs: binding.dispatch.tabs,
        active_tab: binding.dispatch.active_tab,
        categories: binding.dispatch.categories,
        active_category: binding.dispatch.active_category,
        tabs_area: binding.layout.tabs_area,
        category_area: binding.layout.category_area,
    });
    Ok(handled)
}

fn dispatch_overlay_event(
    widget: &mut OverlayRootWidget,
    ctx: &mut EventCtx<'_>,
    event: &crossterm::event::Event,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<EventResult, Box<dyn Error>> {
    match OverlayRootWidget::active_kind(ctx.view) {
        Some(OverlayKind::Summary) => widget.summary.event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::Jump) => widget.jump.event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::Model) => widget.model.event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::Prompt) => widget.prompt.event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::QuestionReview) => widget
            .question_review
            .event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::CodeExec) => widget
            .code_exec
            .event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::FilePatch) => widget
            .file_patch
            .event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::Terminal) => widget.terminal.event(ctx, event, layout, update, jump_rows),
        Some(OverlayKind::Help) => widget.help.event(ctx, event, layout, update, jump_rows),
        None => Ok(EventResult::ignored()),
    }
}
