use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_loop_iteration::IterationSnapshot;
use crate::ui::runtime_loop_steps::{
    DispatchContextParams, LayoutContextParams, note_elapsed, poll_and_dispatch_event,
};
use crate::ui::runtime_render::render_view;
use crate::ui::runtime_view::ViewState;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::time::Instant;

pub(crate) struct RenderIterationParams<'a> {
    pub(crate) terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: usize,
    pub(crate) categories: &'a [String],
    pub(crate) active_category: usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) snapshot: &'a IterationSnapshot,
    pub(crate) view: &'a mut ViewState,
    pub(crate) start_time: Instant,
    pub(crate) startup_elapsed: &'a mut Option<std::time::Duration>,
}

struct RenderContextParts<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    tabs: &'a mut Vec<TabState>,
    active_tab: usize,
    categories: &'a [String],
    active_category: usize,
    theme: &'a RenderTheme,
    registry: &'a crate::model_registry::ModelRegistry,
    prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    snapshot: &'a IterationSnapshot,
}

pub(crate) struct DispatchEventsParams<'a> {
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) categories: &'a mut Vec<String>,
    pub(crate) active_category: &'a mut usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) args: &'a Args,
    pub(crate) snapshot: &'a IterationSnapshot,
    pub(crate) jump_rows: &'a [crate::ui::jump::JumpRow],
    pub(crate) view: &'a mut ViewState,
}

pub(crate) fn dispatch_events(
    params: DispatchEventsParams<'_>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut dispatch_params = DispatchContextParams {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        msg_width: params.snapshot.frame.layout.msg_width,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
    };
    let layout_params = LayoutContextParams {
        size: params.snapshot.frame.size,
        tabs_area: params.snapshot.frame.layout.tabs_area,
        msg_area: params.snapshot.frame.layout.msg_area,
        input_area: params.snapshot.frame.layout.input_area,
        category_area: params.snapshot.frame.layout.category_area,
        view_height: params.snapshot.frame.layout.view_height,
        total_lines: params.snapshot.active_data.total_lines,
    };
    poll_and_dispatch_event(
        &mut dispatch_params,
        layout_params,
        params.view,
        params.jump_rows,
    )
}

pub(crate) fn render_iteration(
    params: RenderIterationParams<'_>,
) -> Result<Vec<crate::ui::jump::JumpRow>, Box<dyn std::error::Error>> {
    let (mut render_ctx, view, start_time, startup_elapsed) =
        split_render_iteration(params);
    let jump_rows = render_view(&mut render_ctx, view)?;
    note_elapsed(start_time, startup_elapsed);
    render_ctx.terminal.hide_cursor()?;
    Ok(jump_rows)
}

fn split_render_iteration<'a>(
    params: RenderIterationParams<'a>,
) -> (RenderContext<'a>, &'a mut ViewState, Instant, &'a mut Option<std::time::Duration>) {
    let RenderIterationParams {
        terminal,
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        snapshot,
        view,
        start_time,
        startup_elapsed,
    } = params;
    let render_ctx = build_render_context(RenderContextParts {
        terminal,
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        snapshot,
    });
    (render_ctx, view, start_time, startup_elapsed)
}

fn build_render_context<'a>(parts: RenderContextParts<'a>) -> RenderContext<'a> {
    let RenderContextParts {
        terminal,
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        snapshot,
    } = parts;
    RenderContext {
        terminal,
        tabs,
        active_tab,
        tab_labels: &snapshot.tab_labels,
        active_tab_pos: snapshot.active_tab_pos,
        categories,
        active_category,
        theme,
        startup_text: snapshot.active_data.startup_text.as_deref(),
        full_area: snapshot.frame.size,
        input_height: snapshot.frame.layout.input_height,
        msg_area: snapshot.frame.layout.msg_area,
        tabs_area: snapshot.frame.layout.tabs_area,
        category_area: snapshot.frame.layout.category_area,
        header_area: snapshot.frame.layout.header_area,
        footer_area: snapshot.frame.layout.footer_area,
        msg_width: snapshot.frame.layout.msg_width,
        text: &snapshot.active_data.text,
        total_lines: snapshot.active_data.total_lines,
        header_note: snapshot.header_note.as_deref(),
        models: &registry.models,
        prompts: &prompt_registry.prompts,
    }
}
