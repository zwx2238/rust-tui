use crate::ui::jump::JumpRow;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_render::{
    build_jump_overlay_rows, render_jump_overlay, render_model_overlay, render_prompt_overlay,
    render_summary_overlay,
};
use crate::ui::overlay_render_base::render_chat_view;
use crate::ui::overlay_render_tool::{
    render_code_exec_overlay, render_file_patch_overlay,
};
use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_view::ViewState;
use crate::ui::shortcut_help::help_rows_len;
use std::error::Error;

pub(crate) struct WidgetFrame<'a, 'b> {
    pub(crate) ctx: &'a mut RenderContext<'b>,
    pub(crate) view: &'a mut ViewState,
    pub(crate) jump_rows: &'a [JumpRow],
}

pub(crate) trait Widget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>>;
}

pub(crate) struct WidgetSystem {
    root: RootWidget,
}

impl WidgetSystem {
    pub(crate) fn new() -> Self {
        Self {
            root: RootWidget::new(),
        }
    }

    pub(crate) fn render<'a>(
        &mut self,
        ctx: &mut RenderContext<'a>,
        view: &mut ViewState,
    ) -> Result<Vec<JumpRow>, Box<dyn Error>> {
        let jump_rows = build_jump_overlay_rows(view, ctx);
        clamp_overlay_tables(view, ctx, jump_rows.len());
        {
            let mut frame = WidgetFrame {
                ctx,
                view,
                jump_rows: &jump_rows,
            };
            self.root.render(&mut frame)?;
        }
        Ok(jump_rows)
    }
}

struct RootWidget {
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
    fn new() -> Self {
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

impl Widget for RootWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        self.render_active(frame)
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

impl Widget for ChatWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_chat_view(frame.ctx)
    }
}

impl Widget for SummaryWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_summary_overlay(frame.ctx, frame.view)
    }
}

impl Widget for JumpWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_jump_overlay(frame.ctx, frame.view, frame.jump_rows)
    }
}

impl Widget for ModelWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_model_overlay(frame.ctx, frame.view)
    }
}

impl Widget for PromptWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_prompt_overlay(frame.ctx, frame.view)
    }
}

impl Widget for CodeExecWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_code_exec_overlay(frame.ctx)
    }
}

impl Widget for FilePatchWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        render_file_patch_overlay(frame.ctx)
    }
}

impl Widget for HelpWidget {
    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        crate::ui::overlay_render::render_help_overlay(frame.ctx, frame.view)
    }
}

fn clamp_overlay_tables(view: &mut ViewState, ctx: &RenderContext<'_>, jump_len: usize) {
    let areas = OverlayAreas {
        full: ctx.full_area,
        msg: ctx.msg_area,
    };
    let counts = OverlayRowCounts {
        tabs: ctx.tabs.len(),
        jump: jump_len,
        models: ctx.models.len(),
        prompts: ctx.prompts.len(),
        help: help_rows_len(),
    };
    let _ = with_active_table_handle(view, areas, counts, |mut handle| handle.clamp());
}
