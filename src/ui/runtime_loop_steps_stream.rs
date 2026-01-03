use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_loop_helpers::handle_pending_command;
use crate::ui::runtime_tick::{
    ActiveFrameData, build_exec_header_note, collect_stream_events, finalize_done_tabs,
    preheat_inactive_tabs, prepare_active_frame, sync_code_exec_overlay, sync_file_patch_overlay,
    update_code_exec_results, update_tab_widths,
};
use crate::ui::runtime_view::ViewState;
use crate::ui::runtime_yolo::auto_finalize_code_exec;
use crate::ui::tool_service::ToolService;
use ratatui::layout::Rect;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub(crate) fn process_stream_updates(
    rx: &mpsc::Receiver<UiEvent>,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    theme: &RenderTheme,
    msg_width: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
    preheat_tx: &mpsc::Sender<crate::ui::runtime_helpers::PreheatTask>,
    view: &mut ViewState,
) -> Result<(), Box<dyn std::error::Error>> {
    let (done_tabs, tool_queue) = collect_stream_events(rx, tabs, theme);
    apply_tool_queue(tabs, registry, args, tx, tool_queue);
    update_code_exec_results(tabs);
    maybe_auto_finalize(tabs, registry, args, tx);
    finalize_done_tabs(tabs, &done_tabs)?;
    update_tab_widths(tabs, msg_width);
    preheat_inactive_tabs(tabs, active_tab, theme, msg_width, preheat_tx);
    sync_overlays_if_needed(tabs, active_tab, view, args);
    Ok(())
}

pub(crate) fn active_frame_data(
    tabs: &mut [TabState],
    active_tab: usize,
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    input_area: Rect,
    startup_elapsed: Option<Duration>,
) -> ActiveFrameData {
    prepare_active_frame(
        &mut tabs[active_tab],
        theme,
        msg_width,
        view_height,
        input_area,
        startup_elapsed,
    )
}

pub(crate) fn handle_pending_line(
    pending_line: Option<String>,
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    if let Some(line) = pending_line {
        if let Some(tab_state) = tabs.get_mut(active_tab) {
            tab_state.app.pending_send = Some(line);
            crate::ui::runtime_dispatch::start_pending_request(registry, args, tx, active_tab, tab_state);
        }
    }
}

pub(crate) fn handle_pending_command_if_any(
    pending_command: Option<crate::ui::state::PendingCommand>,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    session_location: &mut Option<crate::session::SessionLocation>,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    if let Some(cmd) = pending_command {
        handle_pending_command(
            tabs,
            active_tab,
            categories,
            active_category,
            cmd,
            session_location,
            registry,
            prompt_registry,
            args,
            tx,
        );
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
    tx: &mpsc::Sender<UiEvent>,
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
    tx: &mpsc::Sender<UiEvent>,
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
}
