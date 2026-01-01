use crate::args::Args;
use crate::model_registry::build_model_registry;
use crate::render::RenderTheme;
use crate::session::{SessionLocation, load_session, save_session};
use crate::system_prompts::load_prompts;
use crate::types::ROLE_SYSTEM;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{
    PERF_QUESTIONS, PreheatResult, PreheatTask, TabState, collect_session_tabs, start_tab_request,
};
use crate::ui::runtime_loop::run_loop;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn run(
    args: Args,
    cfg: crate::config::Config,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = build_model_registry(&cfg);
    let prompt_registry = load_prompts(&cfg.prompts_dir, "default", &args.system)?;

    let mut session_location: Option<SessionLocation> = None;
    let (mut tabs, mut active_tab) = if let Some(resume) = args.resume.as_deref() {
        let loaded =
            load_session(resume).map_err(|_| format!("无法读取会话：{resume}"))?;
        session_location = Some(loaded.location.clone());
        restore_tabs_from_session(&loaded.data, &registry, &prompt_registry, &args)
    } else {
        let initial_tabs = if args.perf_batch {
            10
        } else if args.perf {
            3
        } else {
            1
        };
        let tabs = (0..initial_tabs)
            .map(|_| {
                TabState::new(
                    prompt_registry
                        .get(&prompt_registry.default_key)
                        .map(|p| p.content.as_str())
                        .unwrap_or(&args.system),
                    args.perf,
                    &registry.default_key,
                    &prompt_registry.default_key,
                )
            })
            .collect::<Vec<_>>();
        (tabs, 0)
    };
    let (tx, rx) = mpsc::channel::<UiEvent>();
    let (preheat_tx, preheat_rx) = mpsc::channel::<PreheatTask>();
    let (preheat_res_tx, preheat_res_rx) = mpsc::channel::<PreheatResult>();
    spawn_preheat_workers(preheat_rx, preheat_res_tx.clone());

    if args.perf_batch && args.resume.is_none() {
        for (i, tab_state) in tabs.iter_mut().enumerate() {
            let question = PERF_QUESTIONS.get(i).unwrap_or(&"请简短说明 Rust 的优势。");
            let model = registry
                .get(&tab_state.app.model_key)
                .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"));
            start_tab_request(
                tab_state,
                question,
                &model.base_url,
                &model.api_key,
                &model.model,
                args.show_reasoning,
                &tx,
                i,
            );
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let start_time = Instant::now();
    run_loop(
        &mut terminal,
        &mut tabs,
        &mut active_tab,
        &mut session_location,
        &rx,
        &tx,
        &preheat_tx,
        &preheat_res_rx,
        &registry,
        &prompt_registry,
        &args,
        theme,
        start_time,
    )?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;

    let snapshot = collect_session_tabs(&tabs);
    if let Ok(loc) = save_session(&snapshot, active_tab, session_location.as_ref()) {
        session_location = Some(loc);
    }
    let dialog_count = tabs.len();
    let message_count: usize = tabs.iter().map(|t| t.app.messages.len()).sum();
    let token_count: u64 = tabs.iter().map(|t| t.app.total_tokens).sum();
    println!(
        "退出统计：对话 {}，消息 {}，token {}",
        dialog_count, message_count, token_count
    );
    if let Some(loc) = session_location {
        println!("恢复指令：deepchat --resume {}", loc.display_hint());
    }

    Ok(())
}

fn restore_tabs_from_session(
    session: &crate::session::SessionData,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::system_prompts::PromptRegistry,
    args: &Args,
) -> (Vec<TabState>, usize) {
    let mut tabs = Vec::new();
    for tab in &session.tabs {
        let model_key = tab
            .model_key
            .as_deref()
            .filter(|k| registry.get(k).is_some())
            .unwrap_or(&registry.default_key)
            .to_string();
        let prompt_key = tab
            .prompt_key
            .as_deref()
            .filter(|k| prompt_registry.get(k).is_some())
            .unwrap_or(&prompt_registry.default_key)
            .to_string();
        let mut state = TabState::new("", false, &model_key, &prompt_key);
        state.app.messages = tab.messages.clone();
        if state.app.messages.iter().all(|m| m.role != ROLE_SYSTEM) {
            let content = prompt_registry
                .get(&prompt_key)
                .map(|p| p.content.as_str())
                .unwrap_or(&args.system);
            if !content.trim().is_empty() {
                state.app.set_system_prompt(&prompt_key, content);
            }
        }
        state.app.follow = true;
        state.app.scroll = u16::MAX;
        state.app.dirty_indices = (0..state.app.messages.len()).collect();
        tabs.push(state);
    }
    if tabs.is_empty() {
        tabs.push(TabState::new(
            prompt_registry
                .get(&prompt_registry.default_key)
                .map(|p| p.content.as_str())
                .unwrap_or(&args.system),
            false,
            &registry.default_key,
            &prompt_registry.default_key,
        ));
    }
    let active_tab = session
        .active_tab
        .min(tabs.len().saturating_sub(1));
    (tabs, active_tab)
}

fn spawn_preheat_workers(
    preheat_rx: mpsc::Receiver<PreheatTask>,
    preheat_res_tx: mpsc::Sender<PreheatResult>,
) {
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
        std::thread::spawn(move || {
            loop {
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
            }
        });
    }
}
