use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::events::{RuntimeEvent, UiEvent};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_tick::{
    ActiveFrameData, build_exec_header_note, collect_stream_events_from_batch, finalize_done_tabs,
    preheat_inactive_tabs, prepare_active_frame, sync_code_exec_overlay,
    sync_file_patch_overlay, sync_question_review_overlay, update_code_exec_results,
    update_tab_widths,
};
use crate::ui::runtime_view::ViewState;
use crate::ui::runtime_yolo::auto_finalize_code_exec;
use crate::ui::tool_service::ToolService;
use ratatui::layout::Rect;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub(crate) struct ProcessStreamUpdatesParams<'a> {
    pub llm_events: &'a mut Vec<UiEvent>,
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: usize,
    pub theme: &'a RenderTheme,
    pub msg_width: usize,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub args: &'a Args,
    pub tx: &'a mpsc::Sender<RuntimeEvent>,
    pub preheat_tx: &'a mpsc::Sender<crate::ui::runtime_helpers::PreheatTask>,
    pub view: &'a mut ViewState,
}

pub(crate) struct ActiveFrameDataParams<'a> {
    pub tabs: &'a mut [TabState],
    pub active_tab: usize,
    pub args: &'a Args,
    pub theme: &'a RenderTheme,
    pub msg_width: usize,
    pub view_height: u16,
    pub input_area: Rect,
    pub startup_elapsed: Option<Duration>,
}

pub(crate) fn process_stream_updates(
    params: ProcessStreamUpdatesParams<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_processed, done_tabs, tool_queue) =
        collect_stream_events_from_batch(params.llm_events, params.tabs, params.theme);
    apply_tool_queue(
        params.tabs,
        params.registry,
        params.args,
        params.tx,
        tool_queue,
    );
    update_code_exec_results(params.tabs);
    maybe_auto_finalize(params.tabs, params.registry, params.args, params.tx);
    finalize_done_tabs(params.tabs, &done_tabs)?;
    update_tab_widths(params.tabs, params.msg_width);
    preheat_inactive_tabs(
        params.tabs,
        params.active_tab,
        params.theme,
        params.msg_width,
        params.preheat_tx,
    );
    sync_overlays_if_needed(params.tabs, params.active_tab, params.view, params.args);
    Ok(())
}

pub(crate) fn active_frame_data(params: ActiveFrameDataParams<'_>) -> ActiveFrameData {
    prepare_active_frame(
        &mut params.tabs[params.active_tab],
        params.args,
        params.theme,
        params.msg_width,
        params.view_height,
        params.input_area,
        params.startup_elapsed,
    )
}

pub(crate) fn handle_pending_line(
    pending_line: Option<String>,
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    if let Some(line) = pending_line
        && let Some(tab_state) = tabs.get_mut(active_tab)
    {
        tab_state.app.pending_send = Some(line);
        crate::ui::runtime_dispatch::start_pending_request(registry, args, tx, tab_state);
    }
}

pub(crate) fn header_note(tabs: &[TabState], categories: &[String]) -> Option<String> {
    build_exec_header_note(tabs, categories)
}

pub(crate) fn note_elapsed(start_time: Instant, startup_elapsed: &mut Option<Duration>) {
    if startup_elapsed.is_none() {
        *startup_elapsed = Some(start_time.elapsed());
    }
}

fn apply_tool_queue(
    tabs: &mut [TabState],
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
    tool_queue: Vec<(usize, Vec<crate::types::ToolCall>)>,
) {
    let tool_service = ToolService::new(registry, args, tx);
    for (tab, calls) in tool_queue {
        if let Some(tab_state) = tabs.get_mut(tab) {
            tool_service.apply_tool_calls(tab_state, tab, &calls);
        }
    }
}

fn maybe_auto_finalize(
    tabs: &mut [TabState],
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    if args.yolo_enabled() {
        auto_finalize_code_exec(tabs, registry, args, tx);
    }
}

fn sync_overlays_if_needed(
    tabs: &mut [TabState],
    active_tab: usize,
    view: &mut ViewState,
    args: &Args,
) {
    if !args.yolo_enabled() {
        sync_code_exec_overlay(tabs, active_tab, view);
        sync_file_patch_overlay(tabs, active_tab, view);
    }
    sync_question_review_overlay(tabs, active_tab, view);
}
