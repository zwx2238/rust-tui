use crate::args::Args;
use crate::llm::prompts::load_prompts;
use crate::model_registry::build_model_registry;
use crate::render::RenderTheme;
use crate::ui::runtime_loop::run_loop;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use runtime_state::{
    Channels, RunState, finalize_session, init_and_spawn_preheat, init_run_state,
    load_question_set_option, maybe_fork_retry, run_initial_requests, sync_active_category,
    validate_args,
};
use runtime_terminal::{ensure_tty_ready, setup_terminal, teardown_terminal};
use std::time::Instant;

mod runtime_state;
mod runtime_terminal;

pub fn run(
    args: Args,
    cfg: crate::config::Config,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = build_model_registry(&cfg);
    let prompt_registry = load_prompts(&cfg.prompts_dir, "default", &args.system)?;
    validate_args(&args)?;
    let question_set = load_question_set_option(&args)?;
    let tavily_api_key = cfg.tavily_api_key.clone();
    run_with_context(
        &args,
        &cfg,
        theme,
        &registry,
        &prompt_registry,
        question_set.as_ref(),
        &tavily_api_key,
    )
}

fn run_with_context(
    args: &Args,
    cfg: &crate::config::Config,
    theme: &RenderTheme,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    question_set: Option<&Vec<String>>,
    tavily_api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = init_run_state(
        args,
        cfg,
        registry,
        prompt_registry,
        question_set,
        tavily_api_key,
    )?;
    let channels = init_and_spawn_preheat();
    let auto_retry = maybe_fork_retry(args, &mut state, registry, prompt_registry);
    sync_active_category(&mut state);
    run_initial_requests(
        question_set,
        auto_retry,
        &mut state,
        registry,
        args,
        &channels.tx,
    );
    run_ui_loop(
        &mut state,
        &channels,
        registry,
        prompt_registry,
        args,
        theme,
    )?;
    finalize_session(&mut state)?;
    Ok(())
}

fn run_ui_loop(
    state: &mut RunState,
    channels: &Channels,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_tty_ready()?;
    let mut terminal = setup_terminal()?;
    run_loop_with_terminal(
        &mut terminal,
        state,
        channels,
        registry,
        prompt_registry,
        args,
        theme,
    )?;
    teardown_terminal(&mut terminal)?;
    Ok(())
}

fn run_loop_with_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut RunState,
    channels: &Channels,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    theme: &RenderTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    run_loop(crate::ui::runtime_loop::RunLoopParams {
        terminal,
        tabs: &mut state.tabs,
        active_tab: &mut state.active_tab,
        categories: &mut state.categories,
        active_category: &mut state.active_category,
        session_location: &mut state.session_location,
        rx: &channels.rx,
        tx: &channels.tx,
        preheat_tx: &channels.preheat_tx,
        preheat_res_rx: &channels.preheat_res_rx,
        registry,
        prompt_registry,
        args,
        theme,
        start_time,
    })
}
