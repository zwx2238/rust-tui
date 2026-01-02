use crate::args::Args;
use crate::model_registry::build_model_registry;
use crate::render::RenderTheme;
use crate::session::{SessionLocation, load_session, save_session};
use crate::system_prompts::load_prompts;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{
    PERF_QUESTIONS, PreheatResult, PreheatTask, TabState, collect_session_tabs, start_tab_request,
};
use crate::ui::runtime_loop::run_loop;
use crate::ui::runtime_session::{
    fork_last_tab_for_retry, restore_tabs_from_session, spawn_preheat_workers,
};
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
use std::time::Instant;

pub fn run(
    args: Args,
    cfg: crate::config::Config,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = build_model_registry(&cfg);
    let prompt_registry = load_prompts(&cfg.prompts_dir, "default", &args.system)?;

    let mut session_location: Option<SessionLocation> = None;
    let tavily_api_key = cfg.tavily_api_key.clone();
    let (mut tabs, mut active_tab) = if let Some(resume) = args.resume.as_deref() {
        let loaded =
            load_session(resume).map_err(|_| format!("无法读取会话：{resume}"))?;
        session_location = Some(loaded.location.clone());
        let (mut tabs, active) =
            restore_tabs_from_session(&loaded.data, &registry, &prompt_registry, &args);
        for tab in &mut tabs {
            tab.app.tavily_api_key = tavily_api_key.clone();
            tab.app.prompts_dir = cfg.prompts_dir.clone();
        }
        (tabs, active)
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
                let mut tab = TabState::new(
                    prompt_registry
                        .get(&prompt_registry.default_key)
                        .map(|p| p.content.as_str())
                        .unwrap_or(&args.system),
                    args.perf,
                    &registry.default_key,
                    &prompt_registry.default_key,
                );
                tab.app.tavily_api_key = tavily_api_key.clone();
                tab.app.prompts_dir = cfg.prompts_dir.clone();
                tab
            })
            .collect::<Vec<_>>();
        (tabs, 0)
    };
    let (tx, rx) = mpsc::channel::<UiEvent>();
    let (preheat_tx, preheat_rx) = mpsc::channel::<PreheatTask>();
    let (preheat_res_tx, preheat_res_rx) = mpsc::channel::<PreheatResult>();
    spawn_preheat_workers(preheat_rx, preheat_res_tx.clone());

    let auto_retry = if args.resume.is_some() && args.replay_fork_last {
        fork_last_tab_for_retry(&mut tabs, &mut active_tab, &registry, &prompt_registry, &args)
    } else {
        None
    };

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
    if let Some((tab_idx, question)) = auto_retry {
        if let Some(tab_state) = tabs.get_mut(tab_idx) {
            let model = registry
                .get(&tab_state.app.model_key)
                .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"));
            start_tab_request(
                tab_state,
                &question,
                &model.base_url,
                &model.api_key,
                &model.model,
                args.show_reasoning,
                &tx,
                tab_idx,
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
