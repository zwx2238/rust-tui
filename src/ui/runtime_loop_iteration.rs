use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use crate::ui::runtime_loop_helpers::{
    HandlePendingCommandIfAnyParams, handle_pending_command_if_any,
};
use crate::ui::runtime_loop_iteration_render::{
    DispatchEventsParams, RenderIterationParams, dispatch_events, render_iteration,
};
use crate::ui::runtime_loop_steps::{
    FrameLayout, ProcessStreamUpdatesParams, active_frame_data, frame_layout, handle_pending_line,
    header_note, prepare_categories, process_stream_updates, tab_labels_and_pos,
};
use crate::ui::runtime_tick::{ActiveFrameData, drain_preheat_results};
use crate::ui::runtime_view::ViewState;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::Instant;

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum LoopControl {
    Continue,
    Break,
}

pub(crate) struct RunLoopIterationParams<'a> {
    pub(crate) terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) categories: &'a mut Vec<String>,
    pub(crate) active_category: &'a mut usize,
    pub(crate) session_location: &'a mut Option<crate::session::SessionLocation>,
    pub(crate) rx: &'a mpsc::Receiver<UiEvent>,
    pub(crate) tx: &'a mpsc::Sender<UiEvent>,
    pub(crate) preheat_tx: &'a mpsc::Sender<PreheatTask>,
    pub(crate) preheat_res_rx: &'a mpsc::Receiver<PreheatResult>,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) args: &'a Args,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) start_time: Instant,
    pub(crate) startup_elapsed: &'a mut Option<std::time::Duration>,
    pub(crate) view: &'a mut ViewState,
}

pub(crate) struct IterationSnapshot {
    pub(crate) frame: FrameLayout,
    pub(crate) active_data: ActiveFrameData,
    pub(crate) tab_labels: Vec<String>,
    pub(crate) active_tab_pos: usize,
    pub(crate) header_note: Option<String>,
}

struct BuildIterationSnapshotParams<'a> {
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
    args: &'a Args,
    theme: &'a RenderTheme,
    startup_elapsed: &'a mut Option<std::time::Duration>,
    view: &'a mut ViewState,
}

pub(crate) fn run_loop_iteration(
    mut params: RunLoopIterationParams<'_>,
) -> Result<LoopControl, Box<dyn std::error::Error>> {
    let snapshot = build_iteration_snapshot_from(&mut params)?;
    handle_pending_actions(&mut params, &snapshot);
    let jump_rows = render_iteration_from(&mut params, &snapshot)?;
    if dispatch_events_from(&mut params, &snapshot, &jump_rows)? {
        return Ok(LoopControl::Break);
    }
    Ok(LoopControl::Continue)
}

fn build_iteration_snapshot_from(
    params: &mut RunLoopIterationParams<'_>,
) -> Result<IterationSnapshot, Box<dyn std::error::Error>> {
    build_iteration_snapshot(BuildIterationSnapshotParams {
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
        args: params.args,
        theme: params.theme,
        startup_elapsed: params.startup_elapsed,
        view: params.view,
    })
}

fn handle_pending_actions(params: &mut RunLoopIterationParams<'_>, snapshot: &IterationSnapshot) {
    handle_pending_line(
        snapshot.active_data.pending_line.clone(),
        params.tabs,
        *params.active_tab,
        params.registry,
        params.args,
        params.tx,
    );
    handle_pending_command_if_any(HandlePendingCommandIfAnyParams {
        pending_command: snapshot.active_data.pending_command,
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
}

fn render_iteration_from(
    params: &mut RunLoopIterationParams<'_>,
    snapshot: &IterationSnapshot,
) -> Result<Vec<crate::ui::jump::JumpRow>, Box<dyn std::error::Error>> {
    render_iteration(RenderIterationParams {
        terminal: params.terminal,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        categories: params.categories,
        active_category: *params.active_category,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        snapshot,
        view: params.view,
        start_time: params.start_time,
        startup_elapsed: params.startup_elapsed,
    })
}

fn dispatch_events_from(
    params: &mut RunLoopIterationParams<'_>,
    snapshot: &IterationSnapshot,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    dispatch_events(DispatchEventsParams {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        snapshot,
        jump_rows,
        view: params.view,
    })
}

fn build_iteration_snapshot(
    mut params: BuildIterationSnapshotParams<'_>,
) -> Result<IterationSnapshot, Box<dyn std::error::Error>> {
    let frame = build_frame_layout(&mut params)?;
    let (tab_labels, active_tab_pos) = prepare_tabs(&mut params);
    run_stream_updates(&mut params, &frame)?;
    let active_data = build_active_frame_data(&mut params, &frame);
    let header_note = header_note(params.tabs, params.categories);
    Ok(IterationSnapshot {
        frame,
        active_data,
        tab_labels,
        active_tab_pos,
        header_note,
    })
}

fn build_frame_layout(
    params: &mut BuildIterationSnapshotParams<'_>,
) -> Result<FrameLayout, Box<dyn std::error::Error>> {
    let frame = frame_layout(
        params.terminal,
        params.view,
        params.tabs,
        *params.active_tab,
        params.categories,
    )?;
    drain_preheat_results(params.preheat_res_rx, params.tabs);
    Ok(frame)
}

fn prepare_tabs(params: &mut BuildIterationSnapshotParams<'_>) -> (Vec<String>, usize) {
    let active_category_name = prepare_categories(
        params.tabs,
        *params.active_tab,
        params.categories,
        params.active_category,
    );
    tab_labels_and_pos(params.tabs, *params.active_tab, &active_category_name)
}

fn run_stream_updates(
    params: &mut BuildIterationSnapshotParams<'_>,
    frame: &FrameLayout,
) -> Result<(), Box<dyn std::error::Error>> {
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
    })
}

fn build_active_frame_data(
    params: &mut BuildIterationSnapshotParams<'_>,
    frame: &FrameLayout,
) -> ActiveFrameData {
    active_frame_data(
        params.tabs,
        *params.active_tab,
        params.theme,
        frame.layout.msg_width,
        frame.layout.view_height,
        frame.layout.input_area,
        *params.startup_elapsed,
    )
}
