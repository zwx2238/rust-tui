use std::error::Error;

use crate::ui::overlay::OverlayKind;
use crate::ui::widget_system::Widget;

use super::chat::ChatWidget;
use super::code_exec::CodeExecWidget;
use super::file_patch::FilePatchWidget;
use super::help::HelpWidget;
use super::jump::JumpWidget;
use super::model::ModelWidget;
use super::prompt::PromptWidget;
use super::summary::SummaryWidget;
use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};

pub(crate) struct RootWidget {
    chat: ChatWidget,
    summary: SummaryWidget,
    jump: JumpWidget,
    model: ModelWidget,
    prompt: PromptWidget,
    code_exec: CodeExecWidget,
    file_patch: FilePatchWidget,
    help: HelpWidget,
}

impl RootWidget {
    pub(crate) fn new() -> Self {
        Self {
            chat: ChatWidget,
            summary: SummaryWidget,
            jump: JumpWidget,
            model: ModelWidget,
            prompt: PromptWidget,
            code_exec: CodeExecWidget,
            file_patch: FilePatchWidget,
            help: HelpWidget,
        }
    }

    fn active_widget(&mut self, active: Option<OverlayKind>) -> &mut dyn Widget {
        match active {
            None => &mut self.chat,
            Some(OverlayKind::Summary) => &mut self.summary,
            Some(OverlayKind::Jump) => &mut self.jump,
            Some(OverlayKind::Model) => &mut self.model,
            Some(OverlayKind::Prompt) => &mut self.prompt,
            Some(OverlayKind::CodeExec) => &mut self.code_exec,
            Some(OverlayKind::FilePatch) => &mut self.file_patch,
            Some(OverlayKind::Help) => &mut self.help,
        }
    }
}

impl Widget for RootWidget {
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
    ) -> Result<crate::ui::runtime_loop_steps::FrameLayout, Box<dyn Error>> {
        let active = ctx.view.overlay.active;
        self.active_widget(active).layout(ctx)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        let active = ctx.view.overlay.active;
        self.active_widget(active).update(ctx, layout)
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
    ) -> Result<bool, Box<dyn Error>> {
        let active = ctx.view.overlay.active;
        self.active_widget(active)
            .event(ctx, layout, update, jump_rows)
    }

    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        let active = frame.view.overlay.active;
        self.active_widget(active).render(frame)
    }
}
