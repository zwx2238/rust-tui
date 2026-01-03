use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use crate::ui::runtime_loop_helpers::{
    HandlePendingCommandIfAnyParams, handle_pending_command_if_any,
};
use crate::ui::runtime_loop_steps::{
    DispatchContextParams, FrameLayout, LayoutContextParams, ProcessStreamUpdatesParams,
    active_frame_data, frame_layout, handle_pending_line, header_note, note_elapsed,
    poll_and_dispatch_event, prepare_categories, process_stream_updates, tab_labels_and_pos,
};
use crate::ui::runtime_render::render_view;
use crate::ui::runtime_tick::drain_preheat_results;
use crate::ui::runtime_view::ViewState;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::Instant;

#[derive(Copy, Clone, Eq, PartialEq)]
enum LoopControl {
    Continue,
    Break,
}

struct IterationData {
    frame: FrameLayout,
    active_data: crate::ui::runtime_tick::ActiveFrameData,
    jump_rows: Vec<crate::ui::jump::JumpRow>,
}

pub(crate) struct RunLoopParams<'a> {
    pub terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: &'a mut usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub session_location: &'a mut Option<crate::session::SessionLocation>,
    pub rx: &'a mpsc::Receiver<UiEvent>,
    pub tx: &'a mpsc::Sender<UiEvent>,
    pub preheat_tx: &'a mpsc::Sender<PreheatTask>,
    pub preheat_res_rx: &'a mpsc::Receiver<PreheatResult>,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
    pub theme: &'a RenderTheme,
    pub start_time: Instant,
}

pub(crate) fn run_loop(params: RunLoopParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let mut startup_elapsed = None;
    let mut view = ViewState::new();
    loop {
        if run_loop_iteration(RunLoopIterationParams {
            terminal: params.terminal,
            tabs: params.tabs,
            active_tab: params.active_tab,
            categories: params.categories,
            active_category: params.active_category,
            session_location: params.session_location,
            rx: params.rx,
            tx: params.tx,
            preheat_tx: params.preheat_tx,
            preheat_res_rx: params.preheat_res_rx,
            registry: params.registry,
            prompt_registry: params.prompt_registry,
            args: params.args,
            theme: params.theme,
            start_time: params.start_time,
            startup_elapsed: &mut startup_elapsed,
            view: &mut view,
        })? == LoopControl::Break
        {
            break;
        }
        #[cfg(test)]
        if std::env::var("DEEPCHAT_TEST_RUN_LOOP_ONCE").is_ok() {
            break;
        }
    }
    Ok(())
}

struct RunLoopIterationParams<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    tabs: &'a mut Vec<TabState>,
    active_tab: &'a mut usize,
    categories: &'a mut Vec<String>,
    active_category: &'a mut usize,
    session_location: &'a mut Option<crate::session::SessionLocation>,
    rx: &'a mpsc::Receiver<UiEvent>,
    tx: &'a mpsc::Sender<UiEvent>,
    preheat_tx: &'a mpsc::Sender<PreheatTask>,
    preheat_res_rx: &'a mpsc::Receiver<PreheatResult>,
    registry: &'a crate::model_registry::ModelRegistry,
    prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    args: &'a Args,
    theme: &'a RenderTheme,
    start_time: Instant,
    startup_elapsed: &'a mut Option<std::time::Duration>,
    view: &'a mut ViewState,
}

fn run_loop_iteration(
    params: RunLoopIterationParams<'_>,
) -> Result<LoopControl, Box<dyn std::error::Error>> {
    let data = build_iteration_data(BuildIterationDataParams {
        terminal: params.terminal,
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        rx: params.rx,
        tx: params.tx,
        preheat_tx: params.preheat_tx,
        preheat_res_rx: params.preheat_res_rx,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        theme: params.theme,
        start_time: params.start_time,
        startup_elapsed: params.startup_elapsed,
        view: params.view,
    })?;
    handle_pending_line(
        data.active_data.pending_line.clone(),
        params.tabs,
        *params.active_tab,
        params.registry,
        params.args,
        params.tx,
    );
    handle_pending_command_if_any(HandlePendingCommandIfAnyParams {
        pending_command: data.active_data.pending_command,
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        session_location: params.session_location,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        tx: params.tx,
    });
    if dispatch_events(DispatchEventsParams {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        data: &data,
        view: params.view,
    })? {
        return Ok(LoopControl::Break);
    }
    Ok(LoopControl::Continue)
}

struct BuildIterationDataParams<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    tabs: &'a mut Vec<TabState>,
    active_tab: &'a mut usize,
    categories: &'a mut Vec<String>,
    active_category: &'a mut usize,
    rx: &'a mpsc::Receiver<UiEvent>,
    tx: &'a mpsc::Sender<UiEvent>,
    preheat_tx: &'a mpsc::Sender<PreheatTask>,
    preheat_res_rx: &'a mpsc::Receiver<PreheatResult>,
    registry: &'a crate::model_registry::ModelRegistry,
    prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    args: &'a Args,
    theme: &'a RenderTheme,
    start_time: Instant,
    startup_elapsed: &'a mut Option<std::time::Duration>,
    view: &'a mut ViewState,
}

struct DispatchEventsParams<'a> {
    tabs: &'a mut Vec<TabState>,
    active_tab: &'a mut usize,
    categories: &'a mut Vec<String>,
    active_category: &'a mut usize,
    theme: &'a RenderTheme,
    registry: &'a crate::model_registry::ModelRegistry,
    prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    args: &'a Args,
    data: &'a IterationData,
    view: &'a mut ViewState,
}

fn build_iteration_data(
    params: BuildIterationDataParams<'_>,
) -> Result<IterationData, Box<dyn std::error::Error>> {
    let frame = frame_layout(
        params.terminal,
        params.view,
        params.tabs,
        *params.active_tab,
        params.categories,
    )?;
    drain_preheat_results(params.preheat_res_rx, params.tabs);
    let active_category_name = prepare_categories(
        params.tabs,
        *params.active_tab,
        params.categories,
        params.active_category,
    );
    let (tab_labels, active_tab_pos) =
        tab_labels_and_pos(params.tabs, *params.active_tab, &active_category_name);
    process_stream_updates(ProcessStreamUpdatesParams {
        rx: params.rx,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        theme: params.theme,
        msg_width: frame.layout.msg_width,
        registry: params.registry,
        args: params.args,
        tx: params.tx,
        preheat_tx: params.preheat_tx,
        view: params.view,
    })?;
    let active_data = active_frame_data(
        params.tabs,
        *params.active_tab,
        params.theme,
        frame.layout.msg_width,
        frame.layout.view_height,
        frame.layout.input_area,
        *params.startup_elapsed,
    );
    let header_note_value = header_note(params.tabs, params.categories);
    let jump_rows = render_frame(RenderFrameParams {
        terminal: params.terminal,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        tab_labels: &tab_labels,
        active_tab_pos,
        categories: params.categories,
        active_category: *params.active_category,
        theme: params.theme,
        startup_text: active_data.startup_text.as_deref(),
        full_area: frame.size,
        input_height: frame.layout.input_height,
        msg_area: frame.layout.msg_area,
        tabs_area: frame.layout.tabs_area,
        category_area: frame.layout.category_area,
        header_area: frame.layout.header_area,
        footer_area: frame.layout.footer_area,
        msg_width: frame.layout.msg_width,
        text: &active_data.text,
        total_lines: active_data.total_lines,
        header_note: header_note_value.as_deref(),
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        view: params.view,
    })?;
    note_elapsed(params.start_time, params.startup_elapsed);
    params.terminal.hide_cursor()?;
    Ok(IterationData {
        frame,
        active_data,
        jump_rows,
    })
}

fn dispatch_events(params: DispatchEventsParams<'_>) -> Result<bool, Box<dyn std::error::Error>> {
    let mut dispatch_params = DispatchContextParams {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        msg_width: params.data.frame.layout.msg_width,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
    };
    let layout_params = LayoutContextParams {
        size: params.data.frame.size,
        tabs_area: params.data.frame.layout.tabs_area,
        msg_area: params.data.frame.layout.msg_area,
        input_area: params.data.frame.layout.input_area,
        category_area: params.data.frame.layout.category_area,
        view_height: params.data.frame.layout.view_height,
        total_lines: params.data.active_data.total_lines,
    };
    poll_and_dispatch_event(
        &mut dispatch_params,
        layout_params,
        params.view,
        &params.data.jump_rows,
    )
}

struct RenderFrameParams<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    tabs: &'a mut Vec<TabState>,
    active_tab: usize,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a Vec<String>,
    active_category: usize,
    theme: &'a RenderTheme,
    startup_text: Option<&'a str>,
    full_area: ratatui::layout::Rect,
    input_height: u16,
    msg_area: ratatui::layout::Rect,
    tabs_area: ratatui::layout::Rect,
    category_area: ratatui::layout::Rect,
    header_area: ratatui::layout::Rect,
    footer_area: ratatui::layout::Rect,
    msg_width: usize,
    text: &'a ratatui::text::Text<'a>,
    total_lines: usize,
    header_note: Option<&'a str>,
    registry: &'a crate::model_registry::ModelRegistry,
    prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    view: &'a mut ViewState,
}

fn render_frame(
    params: RenderFrameParams<'_>,
) -> Result<Vec<crate::ui::jump::JumpRow>, Box<dyn std::error::Error>> {
    let mut render_ctx = RenderContext {
        terminal: params.terminal,
        tabs: params.tabs,
        active_tab: params.active_tab,
        tab_labels: params.tab_labels,
        active_tab_pos: params.active_tab_pos,
        categories: params.categories,
        active_category: params.active_category,
        theme: params.theme,
        startup_text: params.startup_text,
        full_area: params.full_area,
        input_height: params.input_height,
        msg_area: params.msg_area,
        tabs_area: params.tabs_area,
        category_area: params.category_area,
        header_area: params.header_area,
        footer_area: params.footer_area,
        msg_width: params.msg_width,
        text: params.text,
        total_lines: params.total_lines,
        header_note: params.header_note,
        models: &params.registry.models,
        prompts: &params.prompt_registry.prompts,
    };
    render_view(&mut render_ctx, params.view)
}
