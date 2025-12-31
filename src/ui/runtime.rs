use crate::args::Args;
use crate::render::RenderTheme;
use crate::session::save_session;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{
    start_tab_request, PreheatResult, PreheatTask, TabState, PERF_QUESTIONS,
};
use crate::ui::runtime_loop::run_loop;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn run(
    args: Args,
    api_key: String,
    _cfg: Option<crate::config::Config>,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
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
    spawn_preheat_workers(preheat_rx, preheat_res_tx.clone());

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
    run_loop(
        &mut terminal,
        &mut tabs,
        &mut active_tab,
        &mut last_session_id,
        &rx,
        &tx,
        &preheat_tx,
        &preheat_res_rx,
        &url,
        &api_key,
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
}
