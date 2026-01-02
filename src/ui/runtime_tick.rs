use crate::render::{
    RenderTheme, messages_to_viewport_text_cached, messages_to_viewport_text_cached_with_layout,
};
use crate::ui::input_click::update_input_view_top;
use crate::ui::logic::{
    StreamAction, build_label_suffixes, drain_events, handle_stream_event, timer_text,
};
use crate::ui::net::UiEvent;
use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_code_exec_output::build_code_exec_tool_output;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState, enqueue_preheat_tasks};
use crate::ui::runtime_view::ViewState;
use crate::ui::scroll::max_scroll_u16;
use crate::ui::state::PendingCommand;
use ratatui::layout::Rect;
use ratatui::text::Text;
use std::sync::mpsc;
use std::time::Duration;

pub struct ActiveFrameData {
    pub text: Text<'static>,
    pub total_lines: usize,
    pub startup_text: Option<String>,
    pub pending_line: Option<String>,
    pub pending_command: Option<PendingCommand>,
}

pub fn drain_preheat_results(
    preheat_res_rx: &mpsc::Receiver<PreheatResult>,
    tabs: &mut [TabState],
) {
    while let Ok(result) = preheat_res_rx.try_recv() {
        if let Some(tab_state) = tabs.get_mut(result.tab) {
            crate::render::set_cache_entry(&mut tab_state.render_cache, result.idx, result.entry);
        }
    }
}

pub fn collect_stream_events(
    rx: &mpsc::Receiver<UiEvent>,
    tabs: &mut [TabState],
    theme: &RenderTheme,
) -> (Vec<usize>, Vec<(usize, Vec<crate::types::ToolCall>)>) {
    let mut done_tabs: Vec<usize> = Vec::new();
    let mut tool_queue: Vec<(usize, Vec<crate::types::ToolCall>)> = Vec::new();
    while let Ok(event) = rx.try_recv() {
        let UiEvent {
            tab,
            request_id,
            event,
        } = event;
        if let Some(tab_state) = tabs.get_mut(tab) {
            let active_id = tab_state.app.active_request.as_ref().map(|h| h.id);
            if active_id != Some(request_id) {
                continue;
            }
            let elapsed = tab_state
                .app
                .busy_since
                .map(|t| t.elapsed().as_millis() as u64)
                .unwrap_or(0);
            match handle_stream_event(&mut tab_state.app, event, elapsed) {
                StreamAction::Done => {
                    done_tabs.push(tab);
                }
                StreamAction::ToolCalls(calls) => {
                    tool_queue.push((tab, calls));
                }
                StreamAction::None => {}
            }
            tab_state.apply_cache_shift(theme);
        }
    }
    (done_tabs, tool_queue)
}

pub fn finalize_done_tabs(
    tabs: &mut [TabState],
    done_tabs: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    for &tab in done_tabs {
        if let Some(tab_state) = tabs.get_mut(tab) {
            tab_state.app.busy = false;
            tab_state.app.busy_since = None;
        }
    }
    if !done_tabs.is_empty() {
        drain_events()?;
    }
    Ok(())
}

pub fn update_code_exec_results(tabs: &mut [TabState]) {
    for tab_state in tabs.iter_mut() {
        if tab_state.app.pending_code_exec.is_some() && !tab_state.app.code_exec_result_ready {
            let done = tab_state
                .app
                .code_exec_live
                .as_ref()
                .and_then(|live| live.lock().ok().map(|l| l.done))
                .unwrap_or(false);
            if done {
                if let (Some(pending), Some(live)) = (
                    tab_state.app.pending_code_exec.clone(),
                    tab_state.app.code_exec_live.clone(),
                ) {
                    if let Ok(live) = live.lock() {
                        let content = build_code_exec_tool_output(&pending, &live);
                        tab_state.app.code_exec_finished_output = Some(content);
                        tab_state.app.code_exec_result_ready = true;
                    }
                }
            }
        }
    }
}

pub fn update_tab_widths(tabs: &mut [TabState], msg_width: usize) {
    for tab_state in tabs.iter_mut() {
        tab_state.last_width = msg_width;
    }
}

pub fn preheat_inactive_tabs(
    tabs: &mut [TabState],
    active_tab: usize,
    theme: &RenderTheme,
    msg_width: usize,
    preheat_tx: &mpsc::Sender<PreheatTask>,
) {
    for (idx, tab_state) in tabs.iter_mut().enumerate() {
        if idx != active_tab {
            enqueue_preheat_tasks(idx, tab_state, theme, msg_width, 32, preheat_tx);
        }
    }
}

pub fn sync_code_exec_overlay(tabs: &mut [TabState], active_tab: usize, view: &mut ViewState) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let has_pending = tab_state.app.pending_code_exec.is_some();
        if has_pending && view.overlay.is_chat() {
            view.overlay.open(OverlayKind::CodeExec);
        } else if !has_pending && view.overlay.is(OverlayKind::CodeExec) {
            view.overlay.close();
        }
    }
}

pub fn sync_file_patch_overlay(tabs: &mut [TabState], active_tab: usize, view: &mut ViewState) {
    if let Some(tab_state) = tabs.get_mut(active_tab) {
        let has_pending = tab_state.app.pending_file_patch.is_some();
        if has_pending && view.overlay.is_chat() {
            view.overlay.open(OverlayKind::FilePatch);
        } else if !has_pending && view.overlay.is(OverlayKind::FilePatch) {
            view.overlay.close();
        }
    }
}

pub fn prepare_active_frame(
    tab_state: &mut TabState,
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    input_area: Rect,
    startup_elapsed: Option<Duration>,
) -> ActiveFrameData {
    let app = &mut tab_state.app;
    let timer_text = timer_text(app);
    let label_suffixes = build_label_suffixes(&app, &timer_text);
    let prev_scroll = app.scroll;
    let (mut text, computed_total_lines, layouts) = messages_to_viewport_text_cached_with_layout(
        &app.messages,
        msg_width,
        theme,
        &label_suffixes,
        app.pending_assistant,
        app.scroll,
        view_height,
        &mut tab_state.render_cache,
    );
    app.message_layouts = layouts;
    let max_scroll = max_scroll_u16(computed_total_lines, view_height);
    if app.follow {
        app.scroll = max_scroll;
    } else if app.scroll > max_scroll {
        app.scroll = max_scroll;
    }
    if app.scroll != prev_scroll {
        let (retext, _) = messages_to_viewport_text_cached(
            &app.messages,
            msg_width,
            theme,
            &label_suffixes,
            app.pending_assistant,
            app.scroll,
            view_height,
            &mut tab_state.render_cache,
        );
        text = retext;
    }
    update_input_view_top(app, input_area);
    let startup_text = startup_elapsed.map(|d| format!("启动耗时 {:.2}s", d.as_secs_f32()));
    let pending_line = app.pending_send.take();
    let pending_command = app.pending_command.take();
    app.dirty_indices.clear();
    app.cache_shift = None;
    ActiveFrameData {
        text,
        total_lines: computed_total_lines,
        startup_text,
        pending_line,
        pending_command,
    }
}

pub fn build_exec_header_note(tabs: &[TabState], categories: &[String]) -> Option<String> {
    let mut pending_tabs = Vec::new();
    for (idx, tab) in tabs.iter().enumerate() {
        if tab.app.pending_code_exec.is_some() || tab.app.code_exec_live.is_some() {
            let category = if !tab.category.trim().is_empty() {
                tab.category.clone()
            } else {
                categories
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "默认".to_string())
            };
            let pos = crate::ui::runtime_helpers::tab_position_in_category(tabs, &category, idx)
                .unwrap_or(idx);
            pending_tabs.push(format!("{category}/对话{}", pos + 1));
        }
    }
    if pending_tabs.is_empty() {
        return None;
    }
    let list = pending_tabs.join(", ");
    Some(format!("执行中: {} ({})", pending_tabs.len(), list))
}
