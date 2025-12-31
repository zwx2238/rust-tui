use crate::args::Args;
use crate::render::{insert_empty_cache_entry, messages_to_viewport_text_cached, RenderTheme};
use crate::ui::draw::redraw;
use crate::ui::input_click::update_input_view_top;
use crate::ui::logic::{build_label_suffixes, drain_events, format_timer, handle_stream_event};
use crate::ui::net::UiEvent;
use crate::ui::runtime_events::{handle_key_event, handle_mouse_event, handle_paste_event};
use crate::ui::runtime_helpers::{
    enqueue_preheat_tasks, start_tab_request, PreheatResult, PreheatTask, TabState,
};
use crate::ui::runtime_layout::compute_layout;
use crate::ui::summary::{redraw_summary, summary_row_at};
use crossterm::event::{self, Event, MouseEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{sync::mpsc, time::{Duration, Instant}};
pub(crate) fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    last_session_id: &mut Option<String>,
    rx: &mpsc::Receiver<UiEvent>,
    tx: &mpsc::Sender<UiEvent>,
    preheat_tx: &mpsc::Sender<PreheatTask>,
    preheat_res_rx: &mpsc::Receiver<PreheatResult>,
    url: &str,
    api_key: &str,
    args: &Args,
    theme: &RenderTheme,
    start_time: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut startup_elapsed: Option<Duration> = None;
    let mut summary_open = false;
    let mut summary_selected: usize = 0;
    loop {
        let size = terminal.size()?;
        let size = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let layout = compute_layout(size, summary_open, tabs, *active_tab);
        let tabs_area = layout.tabs_area;
        let msg_area = layout.msg_area;
        let input_area = layout.input_area;
        let msg_width = layout.msg_width;
        let view_height = layout.view_height;
        let input_height = layout.input_height;
        while let Ok(result) = preheat_res_rx.try_recv() {
            if let Some(tab_state) = tabs.get_mut(result.tab) {
                crate::render::set_cache_entry(&mut tab_state.render_cache, result.idx, result.entry);
            }
        }

        let mut done_tabs: Vec<usize> = Vec::new();
        while let Ok(event) = rx.try_recv() {
            let UiEvent { tab, event } = event;
            if let Some(tab_state) = tabs.get_mut(tab) {
                let elapsed = tab_state
                    .app
                    .busy_since
                    .map(|t| t.elapsed().as_secs())
                    .unwrap_or(0);
                if handle_stream_event(&mut tab_state.app, event, elapsed) {
                    done_tabs.push(tab);
                }
                if let Some(shift) = tab_state.app.cache_shift.take() {
                    insert_empty_cache_entry(&mut tab_state.render_cache, shift, theme);
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
        let (text, total_lines, tabs_len, startup_text, mut pending_line) = {
            let tabs_len = tabs.len();
            let tab_state = &mut tabs[*active_tab];
            let app = &mut tab_state.app;
            let timer_text = if app.busy {
                let secs = app
                    .busy_since
                    .map(|t| t.elapsed().as_secs())
                    .unwrap_or(0);
                format_timer(secs)
            } else {
                String::new()
            };
            let label_suffixes = build_label_suffixes(&app, &timer_text);
            let prev_scroll = app.scroll;
            tab_state.last_width = msg_width;
            let (mut text, computed_total_lines) = messages_to_viewport_text_cached(
                &app.messages,
                msg_width,
                theme,
                &label_suffixes,
                app.pending_assistant,
                app.scroll,
                view_height,
                &mut tab_state.render_cache,
            );
            let max_scroll = computed_total_lines
                .saturating_sub(view_height as usize)
                .min(u16::MAX as usize) as u16;
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
            let startup_text = startup_elapsed
                .map(|d| format!("启动耗时 {:.2}s", d.as_secs_f32()));
            let pending_line = app.pending_send.take();
            app.dirty_indices.clear();
            app.cache_shift = None;
            (text, computed_total_lines, tabs_len, startup_text, pending_line)
        };
        if summary_open {
            summary_selected = summary_selected.min(tabs_len.saturating_sub(1));
            redraw_summary(
                terminal,
                tabs,
                *active_tab,
                theme,
                startup_text.as_deref(),
                summary_selected,
            )?;
        } else if let Some(tab_state) = tabs.get_mut(*active_tab) {
            redraw(
                terminal,
                &mut tab_state.app,
                theme,
                &text,
                total_lines,
                tabs_len,
                *active_tab,
                startup_text.as_deref(),
                input_height,
            )?;
        }
        if startup_elapsed.is_none() {
            startup_elapsed = Some(start_time.elapsed());
        }
        terminal.hide_cursor()?;
        if let Some(line) = pending_line.take() {
            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                tab_state.app.pending_send = Some(line);
                start_tab_request(
                    tab_state,
                    "",
                    url,
                    api_key,
                    &args.model,
                    args.show_reasoning,
                    tx,
                    *active_tab,
                );
            }
        }
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == crossterm::event::KeyCode::F(1) {
                        summary_open = !summary_open;
                        if summary_open {
                            summary_selected = *active_tab;
                        }
                        continue;
                    }
                    if summary_open {
                        match key.code {
                            crossterm::event::KeyCode::Esc => {
                                summary_open = false;
                                continue;
                            }
                            crossterm::event::KeyCode::Up => {
                                summary_selected = summary_selected.saturating_sub(1);
                                continue;
                            }
                            crossterm::event::KeyCode::Down => {
                                summary_selected = summary_selected.saturating_add(1);
                                continue;
                            }
                            crossterm::event::KeyCode::Enter => {
                                if summary_selected < tabs.len() {
                                    *active_tab = summary_selected;
                                    summary_open = false;
                                }
                                continue;
                            }
                            _ => {}
                        }
                        continue;
                    }
                    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        match key.code {
                            crossterm::event::KeyCode::Char('t') => {
                                tabs.push(TabState::new(&args.system, args.perf));
                                *active_tab = tabs.len().saturating_sub(1);
                                continue;
                            }
                            crossterm::event::KeyCode::Char('w') => {
                                if tabs.len() > 1 {
                                    tabs.remove(*active_tab);
                                    if *active_tab >= tabs.len() {
                                        *active_tab = tabs.len().saturating_sub(1);
                                    }
                                    continue;
                                }
                            }
                            _ => {}
                        }
                    }
                    match key.code {
                        crossterm::event::KeyCode::F(8) => {
                            if !tabs.is_empty() {
                                *active_tab = if *active_tab == 0 {
                                    tabs.len().saturating_sub(1)
                                } else {
                                    *active_tab - 1
                                };
                            }
                            continue;
                        }
                        crossterm::event::KeyCode::F(9) => {
                            if !tabs.is_empty() {
                                *active_tab = (*active_tab + 1) % tabs.len();
                            }
                            continue;
                        }
                        _ => {}
                    }
                    if handle_key_event(
                        key,
                        tabs,
                        *active_tab,
                        last_session_id,
                        msg_width,
                        theme,
                    )? {
                        break;
                    }
                }
                Event::Paste(paste) => {
                    if !summary_open {
                        handle_paste_event(&paste, tabs, *active_tab);
                    }
                }
                Event::Mouse(m) => {
                    if summary_open {
                        if let Some(row) = summary_row_at(msg_area, tabs.len(), m.row) {
                            summary_selected = row.min(tabs.len().saturating_sub(1));
                            if matches!(m.kind, MouseEventKind::Down(_)) && row < tabs.len() {
                                *active_tab = row;
                                summary_open = false;
                            }
                        }
                    } else {
                        handle_mouse_event(
                            m,
                            tabs,
                            active_tab,
                            tabs_area,
                            msg_area,
                            input_area,
                            msg_width,
                            view_height,
                            total_lines,
                            theme,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
