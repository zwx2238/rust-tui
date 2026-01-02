use crate::args::Args;
use crate::render::{
    RenderTheme, messages_to_viewport_text_cached, messages_to_viewport_text_cached_with_layout,
};
use crate::ui::input_click::update_input_view_top;
use crate::ui::logic::{
    StreamAction, build_label_suffixes, drain_events, handle_stream_event, timer_text,
};
use crate::ui::net::UiEvent;
use crate::ui::runtime_dispatch::{
    handle_key_event_loop, handle_mouse_event_loop, start_pending_request,
};
use crate::ui::runtime_context::{make_dispatch_context, make_layout_context};
use crate::ui::runtime_events::handle_paste_event;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState, enqueue_preheat_tasks};
use crate::ui::runtime_layout::compute_layout;
use crate::ui::runtime_code_exec::build_code_exec_tool_output;
use crate::ui::runtime_loop_helpers::{apply_tool_calls, handle_pending_command};
use crate::ui::runtime_render::render_view;
use crate::ui::runtime_view::ViewState;
use crate::ui::scroll::max_scroll_u16;
use crate::ui::overlay::OverlayKind;
use std::sync::mpsc;
use crossterm::event::{self, Event};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::time::{Duration, Instant};

pub(crate) fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    session_location: &mut Option<crate::session::SessionLocation>,
    rx: &mpsc::Receiver<UiEvent>,
    tx: &mpsc::Sender<UiEvent>,
    preheat_tx: &mpsc::Sender<PreheatTask>,
    preheat_res_rx: &mpsc::Receiver<PreheatResult>,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::system_prompts::PromptRegistry,
    args: &Args,
    theme: &RenderTheme,
    start_time: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut startup_elapsed: Option<Duration> = None;
    let mut view = ViewState::new();
    loop {
        let size = terminal.size()?;
        let size = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let layout = compute_layout(size, &view, tabs, *active_tab);
        let header_area = layout.header_area;
        let tabs_area = layout.tabs_area;
        let msg_area = layout.msg_area;
        let input_area = layout.input_area;
        let footer_area = layout.footer_area;
        let msg_width = layout.msg_width;
        let view_height = layout.view_height;
        let input_height = layout.input_height;
        while let Ok(result) = preheat_res_rx.try_recv() {
            if let Some(tab_state) = tabs.get_mut(result.tab) {
                crate::render::set_cache_entry(
                    &mut tab_state.render_cache,
                    result.idx,
                    result.entry,
                );
            }
        }

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
        for (tab, calls) in tool_queue {
            if let Some(tab_state) = tabs.get_mut(tab) {
                apply_tool_calls(tab_state, tab, &calls, registry, args, tx);
            }
        }
        for tab_state in tabs.iter_mut() {
            if tab_state.app.pending_code_exec.is_some()
                && !tab_state.app.code_exec_result_ready
            {
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
        for &tab in &done_tabs {
            if let Some(tab_state) = tabs.get_mut(tab) {
                tab_state.app.busy = false;
                tab_state.app.busy_since = None;
            }
        }
        if !done_tabs.is_empty() {
            drain_events()?;
        }
        for tab_state in tabs.iter_mut() {
            tab_state.last_width = msg_width;
        }
        for (idx, tab_state) in tabs.iter_mut().enumerate() {
            if idx != *active_tab {
                enqueue_preheat_tasks(idx, tab_state, theme, msg_width, 32, preheat_tx);
            }
        }
        if let Some(tab_state) = tabs.get_mut(*active_tab) {
            let has_pending = tab_state.app.pending_code_exec.is_some();
            if has_pending && view.overlay.is_chat() {
                view.overlay.open(OverlayKind::CodeExec);
            } else if !has_pending && view.overlay.is(OverlayKind::CodeExec) {
                view.overlay.close();
            }
        }
        let (text, total_lines, _tabs_len, startup_text, mut pending_line, pending_command) = {
            let tabs_len = tabs.len();
            let tab_state = &mut tabs[*active_tab];
            let app = &mut tab_state.app;
            let timer_text = timer_text(app);
            let label_suffixes = build_label_suffixes(&app, &timer_text);
            let prev_scroll = app.scroll;
            tab_state.last_width = msg_width;
            let (mut text, computed_total_lines, layouts) =
                messages_to_viewport_text_cached_with_layout(
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
            (
                text,
                computed_total_lines,
                tabs_len,
                startup_text,
                pending_line,
                pending_command,
            )
        };
        let header_note = build_exec_header_note(tabs);
        let jump_rows = render_view(
            terminal,
            tabs,
            *active_tab,
            theme,
            startup_text.as_deref(),
            size,
            input_height,
            msg_area,
            tabs_area,
            header_area,
            footer_area,
            msg_width,
            &text,
            total_lines,
            header_note.as_deref(),
            &mut view,
            &registry.models,
            &prompt_registry.prompts,
        )?;
        if startup_elapsed.is_none() {
            startup_elapsed = Some(start_time.elapsed());
        }
        terminal.hide_cursor()?;
        if let Some(line) = pending_line.take() {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                tab_state.app.pending_send = Some(line);
                start_pending_request(registry, args, tx, *active_tab, tab_state);
            }
        }
        if let Some(cmd) = pending_command {
            handle_pending_command(tabs, *active_tab, cmd, session_location, registry, args, tx);
        }
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    let mut ctx = make_dispatch_context(
                        tabs,
                        active_tab,
                        msg_width,
                        theme,
                        registry,
                        prompt_registry,
                        args,
                    );
                    let layout_ctx = make_layout_context(
                        size,
                        tabs_area,
                        msg_area,
                        input_area,
                        view_height,
                        total_lines,
                    );
                    if handle_key_event_loop(key, &mut ctx, layout_ctx, &mut view, &jump_rows)? {
                        break;
                    }
                }
                Event::Paste(paste) => {
                    if view.is_chat() {
                        handle_paste_event(&paste, tabs, *active_tab);
                    }
                }
                Event::Mouse(m) => {
                    let mut ctx = make_dispatch_context(
                        tabs,
                        active_tab,
                        msg_width,
                        theme,
                        registry,
                        prompt_registry,
                        args,
                    );
                    let layout_ctx = make_layout_context(
                        size,
                        tabs_area,
                        msg_area,
                        input_area,
                        view_height,
                        total_lines,
                    );
                    handle_mouse_event_loop(m, &mut ctx, layout_ctx, &mut view, &jump_rows);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn build_exec_header_note(tabs: &[TabState]) -> Option<String> {
    let mut pending_tabs = Vec::new();
    for (idx, tab) in tabs.iter().enumerate() {
        if tab.app.pending_code_exec.is_some() || tab.app.code_exec_live.is_some() {
            pending_tabs.push(idx + 1);
        }
    }
    if pending_tabs.is_empty() {
        return None;
    }
    let list = pending_tabs
        .iter()
        .map(|t| format!("Tab {}", t))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("执行中: {} ({})", pending_tabs.len(), list))
}

// context helpers live in runtime_context
