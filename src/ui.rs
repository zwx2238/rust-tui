mod draw;
mod input;
mod logic;
mod net;
mod perf;
mod state;

use crate::args::Args;
use crate::render::{
    insert_empty_cache_entry, messages_to_viewport_text_cached, RenderCacheEntry, RenderTheme,
};
use crate::session::save_session;
use crate::types::Message;
use crossterm::event::{self, Event, MouseEventKind};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use draw::{inner_height, inner_width, layout_chunks, redraw, scrollbar_area};
use input::handle_key;
use logic::{
    build_label_suffixes, drain_events, format_timer, handle_stream_event, point_in_rect,
    scroll_from_mouse, tab_label_width,
};
use net::{request_llm_stream, UiEvent};
use perf::seed_perf_messages;
use state::{App, Focus};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

struct TabState {
    app: App,
    render_cache: Vec<RenderCacheEntry>,
    last_width: usize,
}

struct PreheatTask {
    tab: usize,
    idx: usize,
    msg: Message,
    width: usize,
    theme: RenderTheme,
    streaming: bool,
}

struct PreheatResult {
    tab: usize,
    idx: usize,
    entry: RenderCacheEntry,
}

impl TabState {
    fn new(system: &str, perf: bool) -> Self {
        let mut app = App::new(system);
        if perf {
            seed_perf_messages(&mut app);
            app.dirty_indices = (0..app.messages.len()).collect();
        }
        Self {
            app,
            render_cache: Vec::new(),
            last_width: 0,
        }
    }
}

pub fn run(
    args: Args,
    api_key: String,
    _cfg: Option<crate::config::Config>,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let base_url = args.base_url.trim_end_matches('/');
    let url = format!("{base_url}/chat/completions");

    let initial_tabs = if args.perf_batch {
        10
    } else if args.perf {
        3
    } else {
        1
    };
    let mut tabs = (0..initial_tabs)
        .map(|_| TabState::new(&args.system, args.perf))
        .collect::<Vec<_>>();
    let mut active_tab: usize = 0;
    let mut last_session_id: Option<String> = None;
    let (tx, rx) = mpsc::channel::<UiEvent>();
    let (preheat_tx, preheat_rx) = mpsc::channel::<PreheatTask>();
    let (preheat_res_tx, preheat_res_rx) = mpsc::channel::<PreheatResult>();
    let workers = std::env::var("PREHEAT_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| (n.get() / 2).max(1))
                .unwrap_or(1)
        });
    let preheat_rx = Arc::new(Mutex::new(preheat_rx));
    for _ in 0..workers {
        let rx = Arc::clone(&preheat_rx);
        let tx = preheat_res_tx.clone();
        std::thread::spawn(move || loop {
            let task = {
                let guard = match rx.lock() {
                    Ok(g) => g,
                    Err(_) => break,
                };
                guard.recv().ok()
            };
            let task = match task {
                Some(t) => t,
                None => break,
            };
            let entry = crate::render::build_cache_entry(
                &task.msg,
                task.width,
                &task.theme,
                task.streaming,
            );
            let _ = tx.send(PreheatResult {
                tab: task.tab,
                idx: task.idx,
                entry,
            });
        });
    }

    if args.perf_batch {
        for (i, tab_state) in tabs.iter_mut().enumerate() {
            let question = PERF_QUESTIONS
                .get(i)
                .unwrap_or(&"请简短说明 Rust 的优势。");
            start_tab_request(
                tab_state,
                question,
                &url,
                &api_key,
                &args.model,
                args.show_reasoning,
                &tx,
                i,
            );
        }
    }

    let start_time = Instant::now();
    let mut startup_elapsed: Option<Duration> = None;

    loop {
        let size = terminal.size()?;
        let (tabs_area, msg_area, input_area) = layout_chunks(size);
        let msg_width = inner_width(msg_area, 1);
        let view_height = inner_height(msg_area, 0) as u16;

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

        for tab_state in &mut tabs {
            tab_state.last_width = msg_width;
        }
        for (idx, tab_state) in tabs.iter_mut().enumerate() {
            if idx != active_tab {
                enqueue_preheat_tasks(idx, tab_state, theme, msg_width, 32, &preheat_tx);
            }
        }

        let pending_send: bool;
        let total_lines = {
            let tabs_len = tabs.len();
            let tab_state = &mut tabs[active_tab];
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
            let startup_text = startup_elapsed
                .map(|d| format!("启动耗时 {:.2}s", d.as_secs_f32()));
            redraw(
                &mut terminal,
                &app,
                theme,
                &text,
                computed_total_lines,
                tabs_len,
                active_tab,
                startup_text.as_deref(),
            )?;
            if startup_elapsed.is_none() {
                startup_elapsed = Some(start_time.elapsed());
            }
            if app.focus == Focus::Input && !app.busy {
                terminal.show_cursor()?;
            } else {
                terminal.hide_cursor()?;
            }

            let should_send = app.pending_send.is_some();
            if should_send {
                app.pending_send.take();
            }
            app.dirty_indices.clear();
            app.cache_shift = None;
            pending_send = should_send;
            computed_total_lines
        };
        if pending_send {
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                start_tab_request(
                    tab_state,
                    "",
                    &url,
                    &api_key,
                    &args.model,
                    args.show_reasoning,
                    &tx,
                    active_tab,
                );
            }
        }
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        match key.code {
                            crossterm::event::KeyCode::Char('t') => {
                                tabs.push(TabState::new(&args.system, args.perf));
                                active_tab = tabs.len().saturating_sub(1);
                                continue;
                            }
                            crossterm::event::KeyCode::Char('w') => {
                                if tabs.len() > 1 {
                                    tabs.remove(active_tab);
                                    if active_tab >= tabs.len() {
                                        active_tab = tabs.len().saturating_sub(1);
                                    }
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }
                    match key.code {
                        crossterm::event::KeyCode::F(8) => {
                            if !tabs.is_empty() {
                                active_tab = if active_tab == 0 {
                                    tabs.len().saturating_sub(1)
                                } else {
                                    active_tab - 1
                                };
                            }
                            continue;
                        }
                        crossterm::event::KeyCode::F(9) => {
                            if !tabs.is_empty() {
                                active_tab = (active_tab + 1) % tabs.len();
                            }
                            continue;
                        }
                        _ => {}
                    }
                    if let Some(tab_state) = tabs.get_mut(active_tab) {
                        if handle_key(key, &mut tab_state.app, &mut last_session_id)? {
                            break;
                        }
                    }
                }
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::Down(_) => {
                        if point_in_rect(m.column, m.row, tabs_area) {
                            if let Some(idx) = tab_index_at(m.column, tabs_area, tabs.len()) {
                                active_tab = idx;
                                continue;
                            }
                        }
                        let scroll_area = scrollbar_area(msg_area);
                        if point_in_rect(m.column, m.row, scroll_area)
                            && total_lines > view_height as usize
                        {
                            if let Some(tab_state) = tabs.get_mut(active_tab) {
                                let app = &mut tab_state.app;
                                app.scrollbar_dragging = true;
                                app.follow = false;
                                app.scroll = scroll_from_mouse(
                                    total_lines,
                                    view_height,
                                    scroll_area,
                                    m.row,
                                );
                                app.focus = Focus::Chat;
                            }
                            continue;
                        }
                        if let Some(tab_state) = tabs.get_mut(active_tab) {
                            let app = &mut tab_state.app;
                            if point_in_rect(m.column, m.row, input_area) {
                                app.focus = Focus::Input;
                                app.cursor = app.input.len();
                            } else if point_in_rect(m.column, m.row, msg_area) {
                                app.focus = Focus::Chat;
                            }
                        }
                    }
                    MouseEventKind::Up(_) => {
                        if let Some(tab_state) = tabs.get_mut(active_tab) {
                            tab_state.app.scrollbar_dragging = false;
                        }
                    }
                    MouseEventKind::Drag(_) => {
                        if let Some(tab_state) = tabs.get_mut(active_tab) {
                            let app = &mut tab_state.app;
                            if app.scrollbar_dragging {
                                let scroll_area = scrollbar_area(msg_area);
                                app.follow = false;
                                app.scroll = scroll_from_mouse(
                                    total_lines,
                                    view_height,
                                    scroll_area,
                                    m.row,
                                );
                                app.focus = Focus::Chat;
                            }
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        if let Some(tab_state) = tabs.get_mut(active_tab) {
                            let app = &mut tab_state.app;
                            app.scroll = app.scroll.saturating_sub(3);
                            app.follow = false;
                            app.focus = Focus::Chat;
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        if let Some(tab_state) = tabs.get_mut(active_tab) {
                            let app = &mut tab_state.app;
                            app.scroll = app.scroll.saturating_add(3);
                            app.follow = false;
                            app.focus = Focus::Chat;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if last_session_id.is_none() {
        if let Some(tab) = tabs.get(active_tab) {
            if let Ok(id) = save_session(&tab.app.messages) {
                last_session_id = Some(id);
            }
        }
    }
    if let Some(id) = last_session_id {
        println!("回放指令：deepchat --resume {id}");
    }

    Ok(())
}

fn tab_index_at(x: u16, area: ratatui::layout::Rect, tabs_len: usize) -> Option<usize> {
    let mut cursor = area.x;
    for i in 0..tabs_len {
        let w = tab_label_width(i);
        let next = cursor.saturating_add(w);
        if x >= cursor && x < next {
            return Some(i);
        }
        cursor = next;
        if i + 1 < tabs_len {
            cursor = cursor.saturating_add(1);
        }
    }
    None
}

fn enqueue_preheat_tasks(
    tab_idx: usize,
    tab: &mut TabState,
    theme: &RenderTheme,
    width: usize,
    limit: usize,
    tx: &mpsc::Sender<PreheatTask>,
) {
    if let Some(shift) = tab.app.cache_shift.take() {
        insert_empty_cache_entry(&mut tab.render_cache, shift, theme);
    }
    let mut remaining = limit;
    while remaining > 0 {
        let idx = match tab.app.dirty_indices.pop() {
            Some(i) => i,
            None => break,
        };
        if let Some(msg) = tab.app.messages.get(idx).cloned() {
            let streaming = tab.app.pending_assistant == Some(idx);
            let _ = tx.send(PreheatTask {
                tab: tab_idx,
                idx,
                msg,
                width,
                theme: theme.clone(),
                streaming,
            });
        }
        remaining -= 1;
    }
}

const PERF_QUESTIONS: [&str; 10] = [
    "用一句话解释什么是借用检查。",
    "用三点说明 async/await 的优势。",
    "写一个最小的 TCP echo 服务器示例。",
    "解释什么是零成本抽象。",
    "给出一个 Rust 中的错误处理最佳实践。",
    "简述 trait 和泛型的关系。",
    "解释生命周期标注的用途。",
    "提供一个并发安全的计数器示例。",
    "列出 3 个常用的性能分析工具。",
    "Rust 在系统编程中的典型应用场景有哪些？",
];

fn start_tab_request(
    tab_state: &mut TabState,
    question: &str,
    url: &str,
    api_key: &str,
    model: &str,
    show_reasoning: bool,
    tx: &mpsc::Sender<UiEvent>,
    tab_id: usize,
) {
    let app = &mut tab_state.app;
    if !question.is_empty() {
        app.messages.push(Message {
            role: "user".to_string(),
            content: question.to_string(),
        });
    } else if let Some(line) = app.pending_send.take() {
        app.messages.push(Message {
            role: "user".to_string(),
            content: line,
        });
    } else {
        return;
    }
    let idx = app.messages.len();
    app.messages.push(Message {
        role: "assistant".to_string(),
        content: String::new(),
    });
    app.busy = true;
    app.busy_since = Some(Instant::now());
    app.pending_assistant = Some(idx);
    app.pending_reasoning = None;
    app.stream_buffer.clear();
    app.follow = true;
    app.dirty_indices.push(idx);
    let messages = app.messages.clone();
    let url = url.to_string();
    let api_key = api_key.to_string();
    let model = model.to_string();
    let tx = tx.clone();
    thread::spawn(move || {
        request_llm_stream(
            &url,
            &api_key,
            &model,
            show_reasoning,
            &messages,
            tx,
            tab_id,
        );
    });
}
