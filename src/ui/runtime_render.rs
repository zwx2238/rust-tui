use crate::ui::jump::JumpRow;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_view::ViewState;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_render::{
    build_jump_overlay_rows, render_help_overlay, render_jump_overlay, render_model_overlay,
    render_prompt_overlay, render_summary_overlay,
};
use crate::ui::overlay_render_base::render_chat_view;
use crate::ui::overlay_render_tool::{render_code_exec_overlay, render_file_patch_overlay};
use std::error::Error;

pub(crate) fn render_view(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let jump_rows = build_jump_overlay_rows(view, ctx);
    match view.overlay.active {
        None => render_chat_view(ctx)?,
        Some(OverlayKind::Summary) => render_summary_overlay(ctx, view)?,
        Some(OverlayKind::Jump) => render_jump_overlay(ctx, view, &jump_rows)?,
        Some(OverlayKind::Model) => render_model_overlay(ctx, view)?,
        Some(OverlayKind::Prompt) => render_prompt_overlay(ctx, view)?,
        Some(OverlayKind::CodeExec) => render_code_exec_overlay(ctx)?,
        Some(OverlayKind::FilePatch) => render_file_patch_overlay(ctx)?,
        Some(OverlayKind::Help) => render_help_overlay(ctx, view)?,
    }
    Ok(jump_rows)
}
