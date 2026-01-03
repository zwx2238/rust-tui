use crate::args::Args;
use crate::llm::prompts::load_prompts;
use crate::model_registry::build_model_registry;
use crate::question_set::load_question_set;
use crate::render::RenderTheme;
use crate::session::{SessionLocation, load_session, save_session};
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{
    PreheatResult, PreheatTask, TabState, collect_open_conversations,
};
use crate::ui::runtime_loop::run_loop;
use crate::ui::runtime_requests::start_tab_request;
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
use std::io::{self, IsTerminal};
use std::sync::mpsc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub fn run(
    args: Args,
    cfg: crate::config::Config,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = build_model_registry(&cfg);
    let prompt_registry = load_prompts(&cfg.prompts_dir, "default", &args.system)?;

    let mut session_location: Option<SessionLocation> = None;
    let tavily_api_key = cfg.tavily_api_key.clone();
    if args.resume.is_some() && args.question_set.is_some() {
        return Err("resume 与 question-set 不能同时使用".into());
    }
    let question_set = if let Some(spec) = args.question_set.as_deref() {
        Some(load_question_set(spec).map_err(|e| format!("问题集加载失败：{e}"))?)
    } else {
        None
    };
    let (mut tabs, mut active_tab, mut categories, mut active_category, log_session_id) =
        if let Some(resume) = args.resume.as_deref() {
            let loaded = load_session(resume).map_err(|_| format!("无法读取会话：{resume}"))?;
            session_location = Some(loaded.location.clone());
            let (tabs, active, categories, active_category) =
                restore_tabs_from_session(&loaded.data, &registry, &prompt_registry, &args)?;
            (
                tabs,
                active,
                categories,
                active_category,
                loaded.data.id.clone(),
            )
        } else {
            let log_session_id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "系统时间异常")?
                .as_secs()
                .to_string();
            let categories = vec!["默认".to_string()];
            let active_category = 0usize;
            let initial_tabs = if let Some(ref questions) = question_set {
                questions.len().max(1)
            } else if args.perf {
                3
            } else {
                1
            };
            let tabs = (0..initial_tabs)
                .map(|_| {
                    let conv_id = crate::conversation::new_conversation_id()
                        .unwrap_or_else(|_| log_session_id.clone());
                    let mut tab = TabState::new(
                        conv_id,
                        categories[active_category].clone(),
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
                    tab.app.set_log_session_id(&log_session_id);
                    tab
                })
                .collect::<Vec<_>>();
            (tabs, 0, categories, active_category, log_session_id)
        };
    for tab in &mut tabs {
        tab.app.tavily_api_key = tavily_api_key.clone();
        tab.app.prompts_dir = cfg.prompts_dir.clone();
        tab.app.set_log_session_id(&log_session_id);
    }
    let (tx, rx) = mpsc::channel::<UiEvent>();
    let (preheat_tx, preheat_rx) = mpsc::channel::<PreheatTask>();
    let (preheat_res_tx, preheat_res_rx) = mpsc::channel::<PreheatResult>();
    spawn_preheat_workers(preheat_rx, preheat_res_tx.clone());

    let auto_retry = if args.resume.is_some() && args.replay_fork_last {
        fork_last_tab_for_retry(
            &mut tabs,
            &mut active_tab,
            &registry,
            &prompt_registry,
            &args,
        )
    } else {
        None
    };
    if let Some(tab_state) = tabs.get(active_tab) {
        if let Some(idx) = categories.iter().position(|c| c == &tab_state.category) {
            active_category = idx;
        }
    }

    if let Some(questions) = question_set {
        for (i, question) in questions.iter().enumerate() {
            if let Some(tab_state) = tabs.get_mut(i) {
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
                    args.web_search_enabled(),
                    args.code_exec_enabled(),
                    args.read_file_enabled(),
                    args.read_code_enabled(),
                    args.modify_file_enabled(),
                    args.log_requests.clone(),
                    tab_state.app.log_session_id.clone(),
                );
            }
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
                args.web_search_enabled(),
                args.code_exec_enabled(),
                args.read_file_enabled(),
                args.read_code_enabled(),
                args.modify_file_enabled(),
                args.log_requests.clone(),
                tab_state.app.log_session_id.clone(),
            );
        }
    }

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("未检测到终端 TTY。请在真实终端运行，或在 CLion 运行/调试配置中勾选 \"Emulate terminal in output console\"。".into());
    }
    ensure_controlling_tty()?;

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
        &mut categories,
        &mut active_category,
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

    for tab in &tabs {
        let _ = crate::conversation::save_conversation(
            &crate::ui::runtime_helpers::tab_to_conversation(tab),
        );
    }
    let open_conversations = collect_open_conversations(&tabs);
    let active_conversation = tabs.get(active_tab).map(|t| t.conversation_id.clone());
    if let Ok(loc) = save_session(
        &categories,
        &open_conversations,
        active_conversation.as_deref(),
        categories.get(active_category).map(|s| s.as_str()),
        session_location.as_ref(),
    ) {
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

#[cfg(unix)]
fn ensure_controlling_tty() -> Result<(), Box<dyn std::error::Error>> {
    if std::fs::File::open("/dev/tty").is_err() {
        return Err("未检测到控制终端 (无法打开 /dev/tty)。CLion Debug 可能未分配 TTY，请改用外部终端运行或启用 \"Run in terminal\"。".into());
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_controlling_tty() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
