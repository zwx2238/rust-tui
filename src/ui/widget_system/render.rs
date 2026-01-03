use crate::ui::jump::JumpRow;
use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_loop_steps::{FrameLayout, note_elapsed};
use crate::ui::runtime_view::ViewState;
use crate::ui::shortcut_help::help_rows_len;
use std::error::Error;

use super::context::{RenderCtx, UpdateOutput, WidgetFrame};
use super::lifecycle::WidgetRender;
use super::root::RootWidget;

pub(crate) fn render_root(
    ctx: &mut RenderCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    root: &mut RootWidget,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let mut render_ctx = build_render_context(ctx, layout, update);
    let jump_rows = render_root_view(&mut render_ctx, ctx.view, root)?;
    note_elapsed(ctx.start_time, ctx.startup_elapsed);
    render_ctx.terminal.hide_cursor()?;
    Ok(jump_rows)
}

pub(crate) fn render_root_view(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
    root: &mut RootWidget,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let jump_rows = crate::ui::overlay_render::build_jump_overlay_rows(view, ctx);
    clamp_overlay_tables(view, ctx, jump_rows.len());
    {
        let mut frame = WidgetFrame {
            ctx,
            view,
            jump_rows: &jump_rows,
        };
        root.render(&mut frame)?;
    }
    Ok(jump_rows)
}

pub(crate) fn clamp_overlay_tables(
    view: &mut ViewState,
    ctx: &RenderContext<'_>,
    jump_len: usize,
) {
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

fn build_render_context<'a>(
    ctx: &'a mut RenderCtx<'a>,
    layout: &FrameLayout,
    update: &UpdateOutput,
) -> RenderContext<'a> {
    RenderContext {
        terminal: ctx.terminal,
        tabs: ctx.tabs,
        active_tab: ctx.active_tab,
        tab_labels: &update.tab_labels,
        active_tab_pos: update.active_tab_pos,
        categories: ctx.categories,
        active_category: ctx.active_category,
        theme: ctx.theme,
        startup_text: update.active_data.startup_text.as_deref(),
        full_area: layout.size,
        input_height: layout.layout.input_height,
        msg_area: layout.layout.msg_area,
        tabs_area: layout.layout.tabs_area,
        category_area: layout.layout.category_area,
        header_area: layout.layout.header_area,
        footer_area: layout.layout.footer_area,
        msg_width: layout.layout.msg_width,
        text: &update.active_data.text,
        total_lines: update.active_data.total_lines,
        header_note: update.header_note.as_deref(),
        models: &ctx.registry.models,
        prompts: &ctx.prompt_registry.prompts,
    }
}
