use crate::ui::draw::redraw_with_overlay;
use crate::ui::file_patch_popup::draw_file_patch_popup;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::overlay_render_base::render_chat_view;
use crate::ui::render_context::RenderContext;
use std::error::Error;

mod code_exec;

pub(super) struct OverlayRenderContext<'a> {
    terminal: &'a mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    tab_state: &'a mut crate::ui::runtime_helpers::TabState,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    theme: &'a crate::render::RenderTheme,
    startup_text: Option<&'a str>,
    input_height: u16,
    text: &'a ratatui::text::Text<'a>,
    total_lines: usize,
    header_note: Option<&'a str>,
}

pub(crate) fn render_code_exec_overlay(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    let pending = ctx
        .tabs
        .get(ctx.active_tab)
        .and_then(|tab_state| tab_state.app.pending_code_exec.clone());
    if let Some(pending) = pending {
        code_exec::render_code_exec_popup(ctx, pending)
    } else {
        render_chat_view(ctx)
    }
}

pub(crate) fn render_file_patch_overlay(ctx: &mut RenderContext<'_>) -> Result<(), Box<dyn Error>> {
    let pending = ctx
        .tabs
        .get(ctx.active_tab)
        .and_then(|tab_state| tab_state.app.pending_file_patch.clone());
    if let Some(pending) = pending {
        render_file_patch_popup(ctx, pending)
    } else {
        render_chat_view(ctx)
    }
}

fn render_file_patch_popup(
    ctx: &mut RenderContext<'_>,
    pending: crate::ui::state::PendingFilePatch,
) -> Result<(), Box<dyn Error>> {
    let Some(mut parts) = split_overlay_context(ctx) else {
        return Ok(());
    };
    let layout = file_patch_layout(&mut parts)?;
    clamp_patch_scroll(parts.tab_state, &pending, layout, parts.theme);
    let scroll = parts.tab_state.app.file_patch_scroll;
    let hover = parts.tab_state.app.file_patch_hover;
    let selection = parts.tab_state.app.file_patch_selection;
    let theme = parts.theme;
    let redraw = build_redraw_params(&mut parts);
    let redraw_result = redraw_with_overlay(redraw, |f| {
        draw_file_patch_popup(
            f,
            f.area(),
            &pending,
            scroll,
            hover,
            selection,
            theme,
        );
    });
    redraw_result?;
    Ok(())
}

pub(super) fn split_overlay_context<'a>(
    ctx: &'a mut RenderContext<'_>,
) -> Option<OverlayRenderContext<'a>> {
    let RenderContext {
        terminal, tabs, active_tab, tab_labels, active_tab_pos, categories, active_category, theme,
        startup_text, input_height, text, total_lines, header_note, ..
    } = ctx;
    let active_tab = *active_tab;
    let tab_state = tabs.get_mut(active_tab)?;
    Some(OverlayRenderContext {
        terminal,
        tab_state,
        tab_labels,
        active_tab_pos: *active_tab_pos,
        categories,
        active_category: *active_category,
        theme,
        startup_text: *startup_text,
        input_height: *input_height,
        text,
        total_lines: *total_lines,
        header_note: *header_note,
    })
}

pub(super) fn build_redraw_params<'a>(
    parts: &'a mut OverlayRenderContext<'_>,
) -> crate::ui::draw::RedrawWithOverlayParams<'a> {
    crate::ui::draw::RedrawWithOverlayParams {
        terminal: parts.terminal,
        app: &mut parts.tab_state.app,
        theme: parts.theme,
        text: parts.text,
        total_lines: parts.total_lines,
        tab_labels: parts.tab_labels,
        active_tab_pos: parts.active_tab_pos,
        categories: parts.categories,
        active_category: parts.active_category,
        startup_text: parts.startup_text,
        input_height: parts.input_height,
        header_note: parts.header_note,
    }
}

fn file_patch_layout(
    parts: &mut OverlayRenderContext<'_>,
) -> Result<crate::ui::file_patch_popup_layout::FilePatchPopupLayout, Box<dyn Error>> {
    let full = parts.terminal.get_frame().area();
    Ok(crate::ui::file_patch_popup_layout::file_patch_popup_layout(full))
}

fn clamp_patch_scroll(
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    layout: crate::ui::file_patch_popup_layout::FilePatchPopupLayout,
    theme: &crate::render::RenderTheme,
) {
    let max_scroll = patch_max_scroll(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        theme,
    );
    if tab_state.app.file_patch_scroll > max_scroll {
        tab_state.app.file_patch_scroll = max_scroll;
    }
}
