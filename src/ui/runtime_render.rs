use crate::ui::jump::JumpRow;
use crate::ui::overlay::OverlayKind;
use crate::ui::overlay_render::{
    build_jump_overlay_rows, render_chat_view, render_code_exec_overlay, render_file_patch_overlay,
    render_help_overlay, render_jump_overlay, render_model_overlay, render_prompt_overlay,
    render_summary_overlay,
};
use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_view::ViewState;
use crate::ui::shortcuts::all_shortcuts;
use std::error::Error;

pub(crate) fn render_view(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let jump_rows = build_jump_overlay_rows(view, ctx);
    let areas = OverlayAreas {
        full: ctx.full_area,
        msg: ctx.msg_area,
    };
    let counts = OverlayRowCounts {
        tabs: ctx.tabs.len(),
        jump: jump_rows.len(),
        models: ctx.models.len(),
        prompts: ctx.prompts.len(),
        help: all_shortcuts().len(),
    };
    let _ = with_active_table_handle(view, areas, counts, |mut handle| {
        handle.clamp();
    });
    match view.overlay.active {
        Some(OverlayKind::Summary) => {
            render_summary_overlay(ctx, view)?;
        }
        Some(OverlayKind::Jump) => {
            render_jump_overlay(ctx, view, &jump_rows)?;
        }
        None => {
            render_chat_view(ctx)?;
        }
        Some(OverlayKind::Model) => {
            render_model_overlay(ctx, view)?;
        }
        Some(OverlayKind::Prompt) => {
            render_prompt_overlay(ctx, view)?;
        }
        Some(OverlayKind::CodeExec) => {
            render_code_exec_overlay(ctx)?;
        }
        Some(OverlayKind::FilePatch) => {
            render_file_patch_overlay(ctx)?;
        }
        Some(OverlayKind::Help) => {
            render_help_overlay(ctx, view)?;
        }
    }
    Ok(jump_rows)
}
