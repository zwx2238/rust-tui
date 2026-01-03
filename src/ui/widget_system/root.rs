use crate::ui::jump::JumpRow;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_render::{
    render_jump_overlay, render_model_overlay, render_prompt_overlay, render_summary_overlay,
};
use crate::ui::overlay_render_base::render_chat_view;
use crate::ui::overlay_render_tool::{
    render_code_exec_overlay, render_file_patch_overlay,
};
use std::error::Error;

use super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::lifecycle::{WidgetLifecycle, WidgetRender};
use super::pipeline::{event_root, layout_root, update_root};

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

    fn render_active(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        match frame.view.overlay.active {
            Some(OverlayKind::Summary) => self.summary.render(frame),
            Some(OverlayKind::Jump) => self.jump.render(frame),
            None => self.chat.render(frame),
            Some(OverlayKind::Model) => self.model.render(frame),
            Some(OverlayKind::Prompt) => self.prompt.render(frame),
            Some(OverlayKind::CodeExec) => self.code_exec.render(frame),
            Some(OverlayKind::FilePatch) => self.file_patch.render(frame),
            Some(OverlayKind::Help) => self.help.render(frame),
        }
    }
}

impl WidgetRender for RootWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        self.render_active(frame)
    }
}

impl WidgetLifecycle for RootWidget {
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>) -> Result<crate::ui::runtime_loop_steps::FrameLayout, Box<dyn Error>> {
        layout_root(ctx)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        update_root(ctx, layout)
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &crate::ui::runtime_loop_steps::FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
    ) -> Result<bool, Box<dyn Error>> {
        event_root(ctx, layout, update, jump_rows)
    }
}

struct ChatWidget;
struct SummaryWidget;
struct JumpWidget;
struct ModelWidget;
struct PromptWidget;
struct CodeExecWidget;
struct FilePatchWidget;
struct HelpWidget;

impl WidgetRender for ChatWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_chat_view(frame.ctx)
    }
}

impl WidgetRender for SummaryWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_summary_overlay(frame.ctx, frame.view)
    }
}

impl WidgetRender for JumpWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_jump_overlay(frame.ctx, frame.view, frame.jump_rows)
    }
}

impl WidgetRender for ModelWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_model_overlay(frame.ctx, frame.view)
    }
}

impl WidgetRender for PromptWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_prompt_overlay(frame.ctx, frame.view)
    }
}

impl WidgetRender for CodeExecWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_code_exec_overlay(frame.ctx)
    }
}

impl WidgetRender for FilePatchWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_file_patch_overlay(frame.ctx)
    }
}

impl WidgetRender for HelpWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        crate::ui::overlay_render::render_help_overlay(frame.ctx, frame.view)
    }
}
