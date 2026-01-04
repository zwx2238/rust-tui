use crate::args::Args;
use crate::llm::prompt_manager::{augment_system, extract_system};
use crate::llm::templates::RigTemplates;
use crate::render::{
    RenderTheme, ViewportRenderParams, messages_to_viewport_text_cached,
    messages_to_viewport_text_cached_with_layout,
};
use crate::types::{Message, ROLE_SYSTEM};
use crate::ui::input_click::update_input_view_top;
use crate::ui::logic::{
    StreamAction, build_label_suffixes, drain_events, handle_stream_event, timer_text,
};
use crate::ui::net::UiEvent;
use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState, enqueue_preheat_tasks};
use crate::ui::runtime_view::ViewState;
use crate::ui::scroll::max_scroll_u16;
use crate::ui::state::PendingCommand;
use ratatui::layout::Rect;
use ratatui::text::Text;
use std::sync::mpsc;
use std::time::Duration;
mod code_exec;
pub fn update_code_exec_results(tabs: &mut [TabState]) {
    code_exec::update_code_exec_results(tabs);
}
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
        handle_stream_event_for_tab(event, tabs, theme, &mut done_tabs, &mut tool_queue);
    }
    (done_tabs, tool_queue)
}

fn handle_stream_event_for_tab(
    ui_event: UiEvent,
    tabs: &mut [TabState],
    theme: &RenderTheme,
    done_tabs: &mut Vec<usize>,
    tool_queue: &mut Vec<(usize, Vec<crate::types::ToolCall>)>,
) {
    let UiEvent {
        tab,
        request_id,
        event,
    } = ui_event;
    let Some(tab_state) = tabs.get_mut(tab) else {
        return;
    };
    if !is_active_request(tab_state, request_id) {
        return;
    }
    let elapsed = elapsed_millis(tab_state);
    match handle_stream_event(&mut tab_state.app, event, elapsed) {
        StreamAction::Done => done_tabs.push(tab),
        StreamAction::ToolCalls(calls) => tool_queue.push((tab, calls)),
        StreamAction::None => {}
    }
    tab_state.apply_cache_shift(theme);
}

fn is_active_request(tab_state: &TabState, request_id: u64) -> bool {
    let active_id = tab_state.app.active_request.as_ref().map(|h| h.id);
    active_id == Some(request_id)
}

fn elapsed_millis(tab_state: &TabState) -> u64 {
    tab_state
        .app
        .busy_since
        .map(|t| t.elapsed().as_millis() as u64)
        .unwrap_or(0)
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
    args: &Args,
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    input_area: Rect,
    startup_elapsed: Option<Duration>,
) -> ActiveFrameData {
    let label_suffixes = build_active_label_suffixes(&tab_state.app);
    let render_messages = build_display_messages(&tab_state.app, args);
    let (text, computed_total_lines) = update_text_and_scroll(
        tab_state,
        &render_messages,
        theme,
        msg_width,
        view_height,
        &label_suffixes,
    );
    update_input_view_top(&mut tab_state.app, input_area);
    let startup_text = format_startup_text(startup_elapsed);
    let (pending_line, pending_command) = take_pending(&mut tab_state.app);
    reset_active_cache(&mut tab_state.app);
    ActiveFrameData {
        text,
        total_lines: computed_total_lines,
        startup_text,
        pending_line,
        pending_command,
    }
}

fn update_text_and_scroll(
    tab_state: &mut TabState,
    messages: &[Message],
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> (Text<'static>, usize) {
    let prev_scroll = tab_state.app.scroll;
    let (mut text, total) =
        render_active_text(tab_state, messages, theme, msg_width, view_height, label_suffixes);
    update_scroll(&mut tab_state.app, total, view_height);
    if tab_state.app.scroll != prev_scroll {
        text = refresh_text_after_scroll(
            tab_state,
            messages,
            theme,
            msg_width,
            view_height,
            label_suffixes,
        );
    }
    (text, total)
}

fn take_pending(app: &mut crate::ui::state::App) -> (Option<String>, Option<PendingCommand>) {
    let pending_line = app.pending_send.take();
    let pending_command = app.pending_command.take();
    (pending_line, pending_command)
}

fn build_active_label_suffixes(app: &crate::ui::state::App) -> Vec<(usize, String)> {
    let timer = timer_text(app);
    build_label_suffixes(app, &timer)
}

fn render_active_text(
    tab_state: &mut TabState,
    messages: &[Message],
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> (Text<'static>, usize) {
    let app = &mut tab_state.app;
    let params = ViewportRenderParams {
        messages,
        width: msg_width,
        theme,
        label_suffixes,
        streaming_idx: app.pending_assistant,
        scroll: app.scroll,
        height: view_height,
    };
    let (text, total, layouts) =
        messages_to_viewport_text_cached_with_layout(params, &mut tab_state.render_cache);
    app.message_layouts = layouts;
    (text, total)
}

fn update_scroll(app: &mut crate::ui::state::App, total_lines: usize, view_height: u16) -> u16 {
    let max_scroll = max_scroll_u16(total_lines, view_height);
    if app.follow || app.scroll > max_scroll {
        app.scroll = max_scroll;
    }
    max_scroll
}

fn refresh_text_after_scroll(
    tab_state: &mut TabState,
    messages: &[Message],
    theme: &RenderTheme,
    msg_width: usize,
    view_height: u16,
    label_suffixes: &[(usize, String)],
) -> Text<'static> {
    let app = &mut tab_state.app;
    let (text, _) = messages_to_viewport_text_cached(
        crate::render::ViewportRenderParams {
            messages,
            width: msg_width,
            theme,
            label_suffixes,
            streaming_idx: app.pending_assistant,
            scroll: app.scroll,
            height: view_height,
        },
        &mut tab_state.render_cache,
    );
    text
}

fn format_startup_text(startup_elapsed: Option<Duration>) -> Option<String> {
    startup_elapsed.map(|d| format!("启动耗时 {:.2}s", d.as_secs_f32()))
}

fn reset_active_cache(app: &mut crate::ui::state::App) {
    app.dirty_indices.clear();
    app.cache_shift = None;
}

pub(crate) fn build_display_messages(app: &crate::ui::state::App, args: &Args) -> Vec<Message> {
    if app.messages.is_empty() {
        return Vec::new();
    }
    let mut messages = app.messages.clone();
    if let Some(msg) = messages.first_mut()
        && msg.role == ROLE_SYSTEM
    {
        if let Some(full) =
            build_full_prompt_for_display(&app.messages, &app.prompts_dir, args)
        {
            if !full.trim().is_empty() {
                msg.content = full;
            }
        }
    }
    messages
}

fn build_full_prompt_for_display(
    messages: &[Message],
    prompts_dir: &str,
    args: &Args,
) -> Option<String> {
    let templates = RigTemplates::load(prompts_dir).ok()?;
    let enabled = enabled_tool_names(args);
    let tools = templates.tool_defs().ok()?;
    let filtered = filter_tools_for_display(tools, &enabled);
    let base_system = augment_system(&extract_system(messages));
    if filtered.is_empty() {
        return Some(base_system);
    }
    templates.render_preamble(&base_system, &filtered).ok()
}

fn enabled_tool_names(args: &Args) -> Vec<&'static str> {
    let mut out = Vec::new();
    if args.web_search_enabled() {
        out.push("web_search");
    }
    if args.code_exec_enabled() {
        out.push("code_exec");
    }
    if args.read_file_enabled() {
        out.push("read_file");
        out.push("list_dir");
    }
    if args.read_code_enabled() {
        out.push("read_code");
    }
    if args.modify_file_enabled() {
        out.push("modify_file");
    }
    out
}

fn filter_tools_for_display(
    tools: Vec<crate::llm::templates::ToolSchema>,
    enabled: &[&str],
) -> Vec<crate::llm::templates::ToolSchema> {
    if enabled.is_empty() {
        return Vec::new();
    }
    tools
        .into_iter()
        .filter(|tool| enabled.iter().any(|name| *name == tool.name))
        .collect()
}

pub fn build_exec_header_note(tabs: &[TabState], categories: &[String]) -> Option<String> {
    let mut pending_tabs = Vec::new();
    for (idx, tab) in tabs.iter().enumerate() {
        if !tab_has_pending_exec(tab) {
            continue;
        }
        let category = tab_category_name(tab, categories);
        let pos = crate::ui::runtime_helpers::tab_position_in_category(tabs, &category, idx)
            .unwrap_or(idx);
        pending_tabs.push(format!("{category}/对话{}", pos + 1));
    }
    if pending_tabs.is_empty() {
        return None;
    }
    let list = pending_tabs.join(", ");
    Some(format!("执行中: {} ({})", pending_tabs.len(), list))
}

fn tab_has_pending_exec(tab: &TabState) -> bool {
    tab.app.pending_code_exec.is_some() || tab.app.code_exec_live.is_some()
}

fn tab_category_name(tab: &TabState, categories: &[String]) -> String {
    if !tab.category.trim().is_empty() {
        return tab.category.clone();
    }
    categories
        .first()
        .cloned()
        .unwrap_or_else(|| "默认".to_string())
}
