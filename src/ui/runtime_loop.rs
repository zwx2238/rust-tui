use crate::args::Args;
use crate::render::{insert_empty_cache_entry, messages_to_viewport_text_cached, RenderTheme};
use crate::ui::input_click::update_input_view_top;
use crate::ui::logic::{build_label_suffixes, drain_events, format_timer, handle_stream_event};
use crate::ui::net::UiEvent;
use crate::ui::runtime_events::{handle_key_event, handle_mouse_event, handle_paste_event};
use crate::ui::runtime_helpers::{
    enqueue_preheat_tasks, start_tab_request, PreheatResult, PreheatTask, TabState,
};
use crate::ui::runtime_layout::compute_layout;
use crate::ui::runtime_render::render_view;
use crate::ui::runtime_view::{
    apply_view_action, handle_view_key, handle_view_mouse, ViewAction, ViewMode, ViewState,
};
use crate::ui::summary::summary_row_at;
use crate::ui::jump::jump_row_at;
use crate::ui::model_popup::model_row_at;
use crate::ui::prompt_popup::{prompt_row_at, prompt_visible_rows};
use crossterm::event::{self, Event};
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
        let layout = compute_layout(size, view.mode, tabs, *active_tab);
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
                    .map(|t| t.elapsed().as_millis() as u64)
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
        let (text, total_lines, _tabs_len, startup_text, mut pending_line) = {
            let tabs_len = tabs.len();
            let tab_state = &mut tabs[*active_tab];
            let app = &mut tab_state.app;
            let timer_text = if app.busy {
                let ms = app
                    .busy_since
                    .map(|t| t.elapsed().as_millis() as u64)
                    .unwrap_or(0);
                format_timer(ms)
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
            msg_width,
            &text,
            total_lines,
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
                let model = resolve_model(registry, &tab_state.app.model_key);
                start_tab_request(
                    tab_state,
                    "",
                    &model.base_url,
                    &model.api_key,
                    &model.model,
                    args.show_reasoning,
                    tx,
                    *active_tab,
                );
            }
        }
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == crossterm::event::KeyCode::F(5) {
                        if let Some(tab_state) = tabs.get_mut(*active_tab) {
                            if !can_change_prompt(&tab_state.app) {
                                tab_state.app.messages.push(crate::types::Message {
                                    role: "assistant".to_string(),
                                    content:
                                        "已开始对话，无法切换系统提示词，请新开 tab。"
                                            .to_string(),
                                });
                                continue;
                            }
                        }
                    }
                    let action = handle_view_key(
                        &mut view,
                        key,
                        tabs.len(),
                        jump_rows.len(),
                        *active_tab,
                    );
                    if matches!(action, ViewAction::CycleModel) {
                        if let Some(tab_state) = tabs.get_mut(*active_tab) {
                            cycle_model(registry, &mut tab_state.app.model_key);
                        }
                        continue;
                    }
                    if key.code == crossterm::event::KeyCode::F(5)
                        && view.mode == ViewMode::Prompt
                    {
                        if let Some(tab_state) = tabs.get_mut(*active_tab) {
                            if let Some(idx) = prompt_registry
                                .prompts
                                .iter()
                                .position(|p| p.key == tab_state.app.prompt_key)
                            {
                                view.prompt_selected = idx;
                                let viewport_rows =
                                    prompt_visible_rows(size, prompt_registry.prompts.len());
                                let max_scroll = prompt_registry
                                    .prompts
                                    .len()
                                    .saturating_sub(viewport_rows)
                                    .max(1)
                                    .saturating_sub(1);
                                if viewport_rows > 0 {
                                    view.prompt_scroll = view
                                        .prompt_selected
                                        .saturating_sub(viewport_rows.saturating_sub(1))
                                        .min(max_scroll);
                                } else {
                                    view.prompt_scroll = 0;
                                }
                            }
                        }
                        continue;
                    }
                    if let ViewAction::SelectModel(idx) = action {
                        if let Some(tab_state) = tabs.get_mut(*active_tab) {
                            if let Some(model) = registry.models.get(idx) {
                                tab_state.app.model_key = model.key.clone();
                            }
                        }
                        continue;
                    }
                    if let ViewAction::SelectPrompt(idx) = action {
                        if let Some(tab_state) = tabs.get_mut(*active_tab) {
                            if can_change_prompt(&tab_state.app) {
                                if let Some(prompt) = prompt_registry.prompts.get(idx) {
                                    tab_state
                                        .app
                                        .set_system_prompt(&prompt.key, &prompt.content);
                                }
                            } else {
                                tab_state.app.messages.push(crate::types::Message {
                                    role: "assistant".to_string(),
                                    content:
                                        "已开始对话，无法切换系统提示词，请新开 tab。"
                                            .to_string(),
                                });
                            }
                        }
                        continue;
                    }
                    if apply_view_action(action, &jump_rows, tabs, active_tab) {
                        continue;
                    }
                    if key.code == crossterm::event::KeyCode::F(4) {
                        if let Some(tab_state) = tabs.get_mut(*active_tab) {
                            if let Some(idx) = registry.index_of(&tab_state.app.model_key) {
                                view.model_selected = idx;
                            }
                        }
                        continue;
                    }
                    if !view.is_chat() {
                        continue;
                    }
                    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        match key.code {
                            crossterm::event::KeyCode::Char('t') => {
                                tabs.push(TabState::new(
                                    prompt_registry
                                        .get(&prompt_registry.default_key)
                                        .map(|p| p.content.as_str())
                                        .unwrap_or(&args.system),
                                    args.perf,
                                    &registry.default_key,
                                    &prompt_registry.default_key,
                                ));
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
                    if view.is_chat() {
                        handle_paste_event(&paste, tabs, *active_tab);
                    }
                }
                Event::Mouse(m) => {
                    if view.is_chat() {
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
                    } else {
                        if view.mode == ViewMode::Jump {
                            match m.kind {
                                crossterm::event::MouseEventKind::ScrollUp => {
                                    view.jump_scroll = view.jump_scroll.saturating_sub(3);
                                }
                                crossterm::event::MouseEventKind::ScrollDown => {
                                    view.jump_scroll = view.jump_scroll.saturating_add(3);
                                }
                                _ => {}
                            }
                        }
                        if view.mode == ViewMode::Prompt {
                            let viewport_rows =
                                prompt_visible_rows(size, prompt_registry.prompts.len());
                            let max_scroll = prompt_registry
                                .prompts
                                .len()
                                .saturating_sub(viewport_rows)
                                .max(1)
                                .saturating_sub(1);
                            match m.kind {
                                crossterm::event::MouseEventKind::ScrollUp => {
                                    view.prompt_scroll = view.prompt_scroll.saturating_sub(3);
                                }
                                crossterm::event::MouseEventKind::ScrollDown => {
                                    view.prompt_scroll = view.prompt_scroll.saturating_add(3);
                                }
                                _ => {}
                            }
                            view.prompt_scroll = view.prompt_scroll.min(max_scroll);
                            if view.prompt_selected < view.prompt_scroll {
                                view.prompt_selected = view.prompt_scroll;
                            }
                        }
                        let row = match view.mode {
                            ViewMode::Summary => summary_row_at(msg_area, tabs.len(), m.row),
                            ViewMode::Jump => {
                                jump_row_at(msg_area, jump_rows.len(), m.row, view.jump_scroll)
                            }
                            ViewMode::Model => model_row_at(
                                size,
                                registry.models.len(),
                                m.column,
                                m.row,
                            ),
                            ViewMode::Prompt => prompt_row_at(
                                size,
                                prompt_registry.prompts.len(),
                                view.prompt_scroll,
                                m.column,
                                m.row,
                            ),
                            ViewMode::Chat => None,
                        };
                        let action = handle_view_mouse(
                            &mut view,
                            row,
                            tabs.len(),
                            jump_rows.len(),
                            m.kind,
                        );
                        if let ViewAction::SelectModel(idx) = action {
                            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                                if let Some(model) = registry.models.get(idx) {
                                    tab_state.app.model_key = model.key.clone();
                                }
                            }
                            continue;
                        }
                        if let ViewAction::SelectPrompt(idx) = action {
                            if let Some(tab_state) = tabs.get_mut(*active_tab) {
                                if can_change_prompt(&tab_state.app) {
                                    if let Some(prompt) = prompt_registry.prompts.get(idx) {
                                        tab_state
                                            .app
                                            .set_system_prompt(&prompt.key, &prompt.content);
                                    }
                                } else {
                                    tab_state.app.messages.push(crate::types::Message {
                                        role: "assistant".to_string(),
                                        content:
                                            "已开始对话，无法切换系统提示词，请新开 tab。"
                                                .to_string(),
                                    });
                                }
                            }
                            continue;
                        }
                        let _ = apply_view_action(action, &jump_rows, tabs, active_tab);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn resolve_model<'a>(
    registry: &'a crate::model_registry::ModelRegistry,
    key: &str,
) -> &'a crate::model_registry::ModelProfile {
    registry
        .get(key)
        .or_else(|| registry.get(&registry.default_key))
        .expect("model registry is empty")
}

fn cycle_model(registry: &crate::model_registry::ModelRegistry, key: &mut String) {
    if registry.models.is_empty() {
        return;
    }
    let idx = registry.index_of(key).unwrap_or(0);
    let next = (idx + 1) % registry.models.len();
    *key = registry.models[next].key.clone();
}

fn can_change_prompt(app: &crate::ui::state::App) -> bool {
    !app.messages.iter().any(|m| m.role == "user")
}
