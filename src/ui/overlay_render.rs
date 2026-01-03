use crate::ui::draw::redraw_with_overlay;
use crate::ui::jump::{JumpRow, build_jump_rows, max_preview_width, redraw_jump};
use crate::ui::model_popup::draw_model_popup;
use crate::ui::prompt_popup::draw_prompt_popup;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_view::ViewState;
use crate::ui::shortcut_help::draw_shortcut_help;
use crate::ui::summary::{RedrawSummaryParams, redraw_summary};
use std::error::Error;

struct OverlayBaseParams<'a> {
    terminal: &'a mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    tab_state: &'a mut crate::ui::runtime_helpers::TabState,
    theme: &'a crate::render::RenderTheme,
    text: &'a ratatui::text::Text<'a>,
    total_lines: usize,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    startup_text: Option<&'a str>,
    input_height: u16,
    header_note: Option<&'a str>,
}

fn build_overlay_params<'a>(
    parts: OverlayBaseParams<'a>,
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

fn overlay_base_params<'a>(ctx: &'a mut RenderContext<'_>) -> Option<OverlayBaseParams<'a>> {
    let RenderContext { terminal, tabs, active_tab, tab_labels, active_tab_pos, categories, active_category, theme, startup_text, input_height, text, total_lines, header_note, .. } = ctx;
    let tab_state = tabs.get_mut(*active_tab)?;
    Some(OverlayBaseParams { terminal, tab_state, theme, text, total_lines: *total_lines, tab_labels, active_tab_pos: *active_tab_pos, categories, active_category: *active_category, startup_text: *startup_text, input_height: *input_height, header_note: *header_note })
}

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
    let rows = redraw_summary(RedrawSummaryParams {
        terminal: ctx.terminal,
        tabs: ctx.tabs,
        active_tab: ctx.active_tab,
        tab_labels: ctx.tab_labels,
        active_tab_pos: ctx.active_tab_pos,
        categories: ctx.categories,
        active_category: ctx.active_category,
        theme: ctx.theme,
        startup_text: ctx.startup_text,
        header_note: ctx.header_note,
        selected_row: view.summary.selected,
        scroll: view.summary.scroll,
        sort: view.summary_sort,
    })?;
    view.summary_order = rows.iter().map(|r| r.tab_index).collect();
    Ok(())
}

pub(crate) fn render_jump_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
    jump_rows: &[JumpRow],
) -> Result<(), Box<dyn Error>> {
    redraw_jump(crate::ui::jump::JumpRedrawParams {
        terminal: ctx.terminal,
        theme: ctx.theme,
        tabs: ctx.tabs,
        active_tab: ctx.active_tab,
        tab_labels: ctx.tab_labels,
        active_tab_pos: ctx.active_tab_pos,
        categories: ctx.categories,
        active_category: ctx.active_category,
        startup_text: ctx.startup_text,
        header_note: ctx.header_note,
        rows: jump_rows,
        selected: view.jump.selected,
        area: ctx.msg_area,
        header_area: ctx.header_area,
        category_area: ctx.category_area,
        tabs_area: ctx.tabs_area,
        footer_area: ctx.footer_area,
        scroll: view.jump.scroll,
    })?;
    Ok(())
}

pub(crate) fn render_model_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    render_model_overlay_inner(ctx, view)
}

pub(crate) fn render_prompt_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    render_prompt_overlay_inner(ctx, view)
}

fn render_model_overlay_inner(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let models = ctx.models;
    let Some(parts) = overlay_base_params(ctx) else {
        return Ok(());
    };
    let selected = view.model.selected;
    let scroll = view.model.scroll;
    let theme = parts.theme;
    let params = build_overlay_params(parts);
    redraw_with_overlay(params, |f| draw_model_popup(f, f.area(), models, selected, scroll, theme))?;
    Ok(())
}

fn render_prompt_overlay_inner(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    let prompts = ctx.prompts;
    let Some(parts) = overlay_base_params(ctx) else {
        return Ok(());
    };
    let selected = view.prompt.selected;
    let scroll = view.prompt.scroll;
    let theme = parts.theme;
    let params = build_overlay_params(parts);
    redraw_with_overlay(params, |f| {
        draw_prompt_popup(f, f.area(), prompts, selected, scroll, theme);
    })?;
    Ok(())
}

pub(crate) fn render_help_overlay(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
) -> Result<(), Box<dyn Error>> {
    if let Some(tab_state) = ctx.tabs.get_mut(ctx.active_tab) {
        redraw_with_overlay(
            crate::ui::draw::RedrawWithOverlayParams {
                terminal: ctx.terminal,
                app: &mut tab_state.app,
                theme: ctx.theme,
                text: ctx.text,
                total_lines: ctx.total_lines,
                tab_labels: ctx.tab_labels,
                active_tab_pos: ctx.active_tab_pos,
                categories: ctx.categories,
                active_category: ctx.active_category,
                startup_text: ctx.startup_text,
                input_height: ctx.input_height,
                header_note: ctx.header_note,
            },
            |f| {
                draw_shortcut_help(f, f.area(), view.help.selected, view.help.scroll, ctx.theme);
            },
        )?;
    }
    Ok(())
}
