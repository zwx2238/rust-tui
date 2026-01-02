use crate::ui::draw::redraw_with_overlay;
use crate::ui::jump::{JumpRow, build_jump_rows, max_preview_width, redraw_jump};
use crate::ui::model_popup::draw_model_popup;
pub(crate) use crate::ui::overlay_render_base::render_chat_view;
pub(crate) use crate::ui::overlay_render_tool::{
    render_code_exec_overlay, render_file_patch_overlay,
};
use crate::ui::prompt_popup::draw_prompt_popup;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_view::ViewState;
use crate::ui::shortcut_help::draw_shortcut_help;
use crate::ui::summary::redraw_summary;
use std::error::Error;

pub(crate) fn build_jump_overlay_rows(view: &ViewState, ctx: &RenderContext<'_>) -> Vec<JumpRow> {
    if !view.overlay.is(crate::ui::overlay::OverlayKind::Jump) {
        return Vec::new();
    }
    ctx.tabs
        .get(ctx.active_tab)
        .map(|tab| {
            build_jump_rows(
                &tab.app.messages,
                ctx.msg_width,
                max_preview_width(ctx.msg_area),
                tab.app.pending_assistant,
            )
        })
        .unwrap_or_default()
}

pub(crate) fn render_summary_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let rows = redraw_summary(
        ctx.terminal,
        ctx.tabs,
        ctx.active_tab,
        ctx.tab_labels,
        ctx.active_tab_pos,
        ctx.categories,
        ctx.active_category,
        ctx.theme,
        ctx.startup_text,
        ctx.header_note,
        view.summary.selected,
        view.summary.scroll,
        view.summary_sort,
    )?;
    view.summary_order = rows.iter().map(|r| r.tab_index).collect();
    Ok(())
}

pub(crate) fn render_jump_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
    jump_rows: &[JumpRow],
) -> Result<(), Box<dyn Error>> {
    redraw_jump(
        ctx.terminal,
        ctx.theme,
        ctx.tabs,
        ctx.active_tab,
        ctx.tab_labels,
        ctx.active_tab_pos,
        ctx.categories,
        ctx.active_category,
        ctx.startup_text,
        ctx.header_note,
        jump_rows,
        view.jump.selected,
        ctx.msg_area,
        ctx.header_area,
        ctx.category_area,
        ctx.tabs_area,
        ctx.footer_area,
        view.jump.scroll,
    )?;
    Ok(())
}

pub(crate) fn render_model_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw_with_overlay(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            ctx.tab_labels,
            ctx.active_tab_pos,
            ctx.categories,
            ctx.active_category,
            ctx.startup_text,
            ctx.input_height,
            |f| {
                draw_model_popup(
                    f,
                    f.area(),
                    ctx.models,
                    view.model.selected,
                    view.model.scroll,
                    ctx.theme,
                );
            },
            ctx.header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_prompt_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw_with_overlay(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            ctx.tab_labels,
            ctx.active_tab_pos,
            ctx.categories,
            ctx.active_category,
            ctx.startup_text,
            ctx.input_height,
            |f| {
                draw_prompt_popup(
                    f,
                    f.area(),
                    ctx.prompts,
                    view.prompt.selected,
                    view.prompt.scroll,
                    ctx.theme,
                );
            },
            ctx.header_note,
        )?;
    }
    Ok(())
}

pub(crate) fn render_help_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw_with_overlay(
            ctx.terminal,
            &mut tab_state.app,
            ctx.theme,
            ctx.text,
            ctx.total_lines,
            ctx.tab_labels,
            ctx.active_tab_pos,
            ctx.categories,
            ctx.active_category,
            ctx.startup_text,
            ctx.input_height,
            |f| {
                draw_shortcut_help(f, f.area(), view.help.selected, view.help.scroll, ctx.theme);
            },
            ctx.header_note,
        )?;
    }
    Ok(())
}
