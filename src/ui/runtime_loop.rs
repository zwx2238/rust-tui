use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::render_context::RenderContext;
use crate::ui::runtime_context::{make_dispatch_context, make_layout_context};
use crate::ui::runtime_dispatch::{
    handle_key_event_loop, handle_mouse_event_loop, start_pending_request,
};
use crate::ui::runtime_events::handle_paste_event;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use crate::ui::runtime_layout::compute_layout;
use crate::ui::runtime_loop_helpers::handle_pending_command;
use crate::ui::runtime_render::render_view;
use crate::ui::runtime_tick::{
    ActiveFrameData, build_exec_header_note, collect_stream_events, drain_preheat_results,
    finalize_done_tabs, preheat_inactive_tabs, prepare_active_frame, sync_code_exec_overlay,
    sync_file_patch_overlay, update_code_exec_results, update_tab_widths,
};
use crate::ui::runtime_view::ViewState;
use crate::ui::tool_service::ToolService;
use crossterm::event::{self, Event};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub(crate) fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    session_location: &mut Option<crate::session::SessionLocation>,
    rx: &mpsc::Receiver<UiEvent>,
    tx: &mpsc::Sender<UiEvent>,
    preheat_tx: &mpsc::Sender<PreheatTask>,
    preheat_res_rx: &mpsc::Receiver<PreheatResult>,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    theme: &RenderTheme,
    start_time: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut startup_elapsed: Option<Duration> = None;
    let mut view = ViewState::new();
    loop {
        let size = terminal.size()?;
        let size = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let layout = compute_layout(size, &view, tabs, *active_tab, categories);
        let header_area = layout.header_area;
        let category_area = layout.category_area;
        let tabs_area = layout.tabs_area;
        let msg_area = layout.msg_area;
        let input_area = layout.input_area;
        let footer_area = layout.footer_area;
        let msg_width = layout.msg_width;
        let view_height = layout.view_height;
        let input_height = layout.input_height;
        drain_preheat_results(preheat_res_rx, tabs);
        if categories.is_empty() {
            categories.push("默认".to_string());
        }
        if *active_category >= categories.len() {
            *active_category = 0;
        }
        if let Some(tab_state) = tabs.get(*active_tab) {
            if let Some(idx) = categories.iter().position(|c| c == &tab_state.category) {
                *active_category = idx;
            }
        }
        let active_category_name = categories[*active_category].clone();
        let tab_labels =
            crate::ui::runtime_helpers::tab_labels_for_category(tabs, &active_category_name);
        let active_tab_pos = crate::ui::runtime_helpers::active_tab_position(
            tabs,
            &active_category_name,
            *active_tab,
        );

        let (done_tabs, tool_queue) = collect_stream_events(rx, tabs, theme);
        let tool_service = ToolService::new(registry, args, tx);
        for (tab, calls) in tool_queue {
            if let Some(tab_state) = tabs.get_mut(tab) {
                tool_service.apply_tool_calls(tab_state, tab, &calls);
            }
        }
        update_code_exec_results(tabs);
        finalize_done_tabs(tabs, &done_tabs)?;
        update_tab_widths(tabs, msg_width);
        preheat_inactive_tabs(tabs, *active_tab, theme, msg_width, preheat_tx);
        sync_code_exec_overlay(tabs, *active_tab, &mut view);
        sync_file_patch_overlay(tabs, *active_tab, &mut view);
        let ActiveFrameData {
            text,
            total_lines,
            startup_text,
            mut pending_line,
            pending_command,
        } = prepare_active_frame(
            &mut tabs[*active_tab],
            theme,
            msg_width,
            view_height,
            input_area,
            startup_elapsed,
        );
        let header_note = build_exec_header_note(tabs, categories);
        let mut render_ctx = RenderContext {
            terminal,
            tabs,
            active_tab: *active_tab,
            tab_labels: &tab_labels,
            active_tab_pos,
            categories,
            active_category: *active_category,
            theme,
            startup_text: startup_text.as_deref(),
            full_area: size,
            input_height,
            msg_area,
            tabs_area,
            category_area,
            header_area,
            footer_area,
            msg_width,
            text: &text,
            total_lines,
            header_note: header_note.as_deref(),
            models: &registry.models,
            prompts: &prompt_registry.prompts,
        };
        let jump_rows = render_view(&mut render_ctx, &mut view)?;
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
            handle_pending_command(
                tabs,
                *active_tab,
                categories,
                *active_category,
                cmd,
                session_location,
                registry,
                args,
                tx,
            );
        }
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    let mut ctx = make_dispatch_context(
                        tabs,
                        active_tab,
                        categories,
                        active_category,
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
                        category_area,
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
                        categories,
                        active_category,
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
                        category_area,
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

// context helpers live in runtime_context
