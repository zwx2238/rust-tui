use crate::args::Args;
use crate::model_registry::ModelRegistry;
use crate::question_set::load_question_set;
use crate::session::{SessionLocation, load_session, save_session};
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_helpers::{PreheatTask, TabState, collect_open_conversations};
use crate::ui::runtime_session::{
    fork_last_tab_for_retry, restore_tabs_from_session, spawn_preheat_workers,
};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

mod requests;
pub(crate) use requests::run_initial_requests;

pub(crate) struct RunState {
    pub(crate) tabs: Vec<TabState>,
    pub(crate) active_tab: usize,
    pub(crate) categories: Vec<String>,
    pub(crate) active_category: usize,
    pub(crate) log_session_id: String,
    pub(crate) session_location: Option<SessionLocation>,
}

pub(crate) struct Channels {
    pub(crate) tx: mpsc::Sender<RuntimeEvent>,
    pub(crate) rx: mpsc::Receiver<RuntimeEvent>,
    pub(crate) preheat_tx: mpsc::Sender<PreheatTask>,
}

pub(crate) fn validate_args(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.resume.is_some() && args.question_set.is_some() {
        return Err("resume 与 question-set 不能同时使用".into());
    }
    Ok(())
}

pub(crate) fn load_question_set_option(
    args: &Args,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    if let Some(spec) = args.question_set.as_deref() {
        let qs = load_question_set(spec).map_err(|e| format!("问题集加载失败：{e}"))?;
        return Ok(Some(qs));
    }
    Ok(None)
}

pub(crate) fn init_run_state(
    args: &Args,
    cfg: &crate::config::Config,
    registry: &ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    question_set: Option<&Vec<String>>,
    tavily_api_key: &str,
) -> Result<RunState, Box<dyn std::error::Error>> {
    if let Some(resume) = args.resume.as_deref() {
        return load_run_state(resume, registry, prompt_registry, args, cfg, tavily_api_key);
    }
    build_new_state(
        args,
        cfg,
        registry,
        prompt_registry,
        question_set,
        tavily_api_key,
    )
}

fn load_run_state(
    resume: &str,
    registry: &ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    cfg: &crate::config::Config,
    tavily_api_key: &str,
) -> Result<RunState, Box<dyn std::error::Error>> {
    let loaded = load_session(resume).map_err(|_| format!("无法读取会话：{resume}"))?;
    let (tabs, active, categories, active_category) =
        restore_tabs_from_session(&loaded.data, registry, prompt_registry, args)?;
    let mut state = RunState {
        tabs,
        active_tab: active,
        categories,
        active_category,
        log_session_id: loaded.data.id.clone(),
        session_location: Some(loaded.location.clone()),
    };
    apply_tab_config(&mut state.tabs, cfg, tavily_api_key, &state.log_session_id);
    Ok(state)
}

fn build_new_state(
    args: &Args,
    cfg: &crate::config::Config,
    registry: &ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    question_set: Option<&Vec<String>>,
    tavily_api_key: &str,
) -> Result<RunState, Box<dyn std::error::Error>> {
    let log_session_id = new_log_session_id()?;
    let (categories, active_category) = default_categories();
    let initial_tabs = initial_tab_count(args, question_set);
    let tabs = build_initial_tabs(
        initial_tabs,
        &categories,
        active_category,
        prompt_registry,
        args,
        registry,
        &log_session_id,
    );
    let mut state = create_state(tabs, categories, active_category, log_session_id, None);
    apply_tab_config(&mut state.tabs, cfg, tavily_api_key, &state.log_session_id);
    Ok(state)
}

fn default_categories() -> (Vec<String>, usize) {
    (vec!["默认".to_string()], 0)
}

fn create_state(
    tabs: Vec<TabState>,
    categories: Vec<String>,
    active_category: usize,
    log_session_id: String,
    session_location: Option<SessionLocation>,
) -> RunState {
    RunState {
        tabs,
        active_tab: 0,
        categories,
        active_category,
        log_session_id,
        session_location,
    }
}

fn new_log_session_id() -> Result<String, Box<dyn std::error::Error>> {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间异常")?
        .as_secs()
        .to_string();
    Ok(id)
}

fn initial_tab_count(args: &Args, question_set: Option<&Vec<String>>) -> usize {
    if let Some(questions) = question_set {
        return questions.len().max(1);
    }
    if args.perf {
        return 3;
    }
    1
}

fn build_initial_tabs(
    count: usize,
    categories: &[String],
    active_category: usize,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    registry: &ModelRegistry,
    log_session_id: &str,
) -> Vec<TabState> {
    (0..count)
        .map(|_| {
            let conv_id = crate::conversation::new_conversation_id()
                .unwrap_or_else(|_| log_session_id.to_string());
            let prompt = prompt_registry
                .get(&prompt_registry.default_key)
                .map(|p| p.content.as_str())
                .unwrap_or(&args.system);
            TabState::new(
                conv_id,
                categories[active_category].clone(),
                prompt,
                args.perf,
                &registry.default_key,
                &prompt_registry.default_key,
            )
        })
        .collect()
}

fn apply_tab_config(
    tabs: &mut [TabState],
    cfg: &crate::config::Config,
    tavily_api_key: &str,
    log_session_id: &str,
) {
    for tab in tabs {
        tab.app.tavily_api_key = tavily_api_key.to_string();
        tab.app.prompts_dir = cfg.prompts_dir.clone();
        tab.app.set_log_session_id(log_session_id);
    }
}

pub(crate) fn init_and_spawn_preheat() -> Channels {
    init_channels()
}

fn init_channels() -> Channels {
    let (tx, rx) = mpsc::channel::<RuntimeEvent>();
    let (preheat_tx, preheat_rx) = mpsc::channel::<PreheatTask>();
    spawn_preheat_workers(preheat_rx, tx.clone());
    Channels { tx, rx, preheat_tx }
}

pub(crate) fn maybe_fork_retry(
    args: &Args,
    state: &mut RunState,
    registry: &ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
) -> Option<(usize, String)> {
    if args.resume.is_some() && args.replay_fork_last {
        return fork_last_tab_for_retry(
            &mut state.tabs,
            &mut state.active_tab,
            registry,
            prompt_registry,
            args,
        );
    }
    None
}

pub(crate) fn sync_active_category(state: &mut RunState) {
    if let Some(tab_state) = state.tabs.get(state.active_tab)
        && let Some(idx) = state
            .categories
            .iter()
            .position(|c| c == &tab_state.category)
    {
        state.active_category = idx;
    }
}

pub(crate) fn finalize_session(state: &mut RunState) -> Result<(), Box<dyn std::error::Error>> {
    for tab in &state.tabs {
        let _ = crate::conversation::save_conversation(
            &crate::ui::runtime_helpers::tab_to_conversation(tab),
        );
    }
    let open_conversations = collect_open_conversations(&state.tabs);
    let active_conversation = state
        .tabs
        .get(state.active_tab)
        .map(|t| t.conversation_id.clone());
    if let Ok(loc) = save_session(
        &state.categories,
        &open_conversations,
        active_conversation.as_deref(),
        state
            .categories
            .get(state.active_category)
            .map(|s| s.as_str()),
        state.session_location.as_ref(),
    ) {
        state.session_location = Some(loc);
    }
    print_exit_stats(state);
    Ok(())
}

fn print_exit_stats(state: &RunState) {
    let dialog_count = state.tabs.len();
    let message_count: usize = state.tabs.iter().map(|t| t.app.messages.len()).sum();
    let token_count: u64 = state.tabs.iter().map(|t| t.app.total_tokens).sum();
    println!(
        "退出统计：对话 {}，消息 {}，token {}",
        dialog_count, message_count, token_count
    );
    if let Some(loc) = &state.session_location {
        println!("恢复指令：deepchat --resume {}", loc.display_hint());
    }
}
