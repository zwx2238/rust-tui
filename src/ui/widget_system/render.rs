use crate::ui::jump::JumpRow;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_loop_steps::{FrameLayout, note_elapsed};
use crate::ui::runtime_view::ViewState;
use std::error::Error;

use super::context::{RenderCtx, UpdateOutput, WidgetFrame};
use super::lifecycle::Widget;
use super::widgets::RootWidget;

pub(crate) fn render_root<'a>(
    ctx: &'a mut RenderCtx<'a>,
    layout: &'a FrameLayout,
    update: &'a UpdateOutput,
    root: &mut RootWidget,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let (mut render_ctx, view, start_time, startup_elapsed) =
        split_render_inputs(ctx, layout, update);
    let jump_rows = render_root_view(&mut render_ctx, view, root)?;
    note_elapsed(start_time, startup_elapsed);
    render_ctx.terminal.hide_cursor()?;
    Ok(jump_rows)
}

pub(crate) fn render_root_view(
    ctx: &mut RenderContext<'_>,
    view: &mut ViewState,
    root: &mut RootWidget,
) -> Result<Vec<JumpRow>, Box<dyn Error>> {
    let mut jump_rows = Vec::new();
    {
        let mut frame = WidgetFrame {
            ctx,
            view,
            jump_rows: &mut jump_rows,
        };
        root.render(&mut frame)?;
    }
    Ok(jump_rows)
}

fn split_render_inputs<'a>(
    ctx: &'a mut RenderCtx<'a>,
    layout: &'a FrameLayout,
    update: &'a UpdateOutput,
) -> (
    RenderContext<'a>,
    &'a mut ViewState,
    std::time::Instant,
    &'a mut Option<std::time::Duration>,
) {
    let (parts, view, start_time, startup_elapsed) =
        extract_render_parts(ctx, layout, update);
    (
        build_render_context(parts),
        view,
        start_time,
        startup_elapsed,
    )
}

fn extract_render_parts<'a>(
    ctx: &'a mut RenderCtx<'a>,
    layout: &'a FrameLayout,
    update: &'a UpdateOutput,
) -> (RenderContextParts<'a>, &'a mut ViewState, std::time::Instant, &'a mut Option<std::time::Duration>) {
    let RenderCtx { terminal, tabs, active_tab, categories, active_category, theme, registry, prompt_registry, view, start_time, startup_elapsed } = ctx;
    let parts = RenderContextParts { terminal, tabs, active_tab: *active_tab, categories, active_category: *active_category, theme, registry, prompt_registry, layout, update };
    (parts, view, *start_time, startup_elapsed)
}

struct RenderContextParts<'a> {
    terminal: &'a mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    tabs: &'a mut Vec<crate::ui::runtime_helpers::TabState>,
    active_tab: usize,
    categories: &'a [String],
    active_category: usize,
    theme: &'a crate::render::RenderTheme,
    registry: &'a crate::model_registry::ModelRegistry,
    prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    layout: &'a FrameLayout,
    update: &'a UpdateOutput,
}

fn build_render_context<'a>(parts: RenderContextParts<'a>) -> RenderContext<'a> {
    RenderContext {
        terminal: parts.terminal,
        tabs: parts.tabs,
        active_tab: parts.active_tab,
        tab_labels: &parts.update.tab_labels,
        active_tab_pos: parts.update.active_tab_pos,
        categories: parts.categories,
        active_category: parts.active_category,
        theme: parts.theme,
        startup_text: parts.update.active_data.startup_text.as_deref(),
        full_area: parts.layout.size,
        input_height: parts.layout.layout.input_height,
        msg_area: parts.layout.layout.msg_area,
        tabs_area: parts.layout.layout.tabs_area,
        category_area: parts.layout.layout.category_area,
        header_area: parts.layout.layout.header_area,
        footer_area: parts.layout.layout.footer_area,
        msg_width: parts.layout.layout.msg_width,
        text: &parts.update.active_data.text,
        total_lines: parts.update.active_data.total_lines,
        header_note: parts.update.header_note.as_deref(),
        models: &parts.registry.models,
        prompts: &parts.prompt_registry.prompts,
    }
}
