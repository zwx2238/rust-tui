use crate::args::Args;
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_helpers::TabState;
use std::sync::mpsc;

pub(super) struct QuestionTabSpawnParams<'a> {
    pub(super) tabs: &'a mut Vec<TabState>,
    pub(super) active_tab: usize,
    pub(super) categories: &'a mut Vec<String>,
    pub(super) active_category: &'a mut usize,
    pub(super) registry: &'a crate::model_registry::ModelRegistry,
    pub(super) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(super) args: &'a Args,
    pub(super) tx: &'a mpsc::Sender<RuntimeEvent>,
}

pub(super) fn spawn_question_tabs(params: QuestionTabSpawnParams<'_>, approved: &[String]) {
    let Some(seed) = seed_from_active(
        params.tabs,
        params.active_tab,
        params.registry,
        params.prompt_registry,
        params.args,
    ) else {
        return;
    };
    ensure_category(params.categories, params.active_category, &seed.category);
    for question in approved {
        let tab_idx = create_question_tab(params.tabs, &seed);
        start_question_request(
            params.tabs.as_mut_slice(),
            tab_idx,
            question,
            params.args,
            params.tx,
            params.registry,
        );
    }
}

fn seed_from_active(
    tabs: &[TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) -> Option<QuestionTabSeed> {
    let tab_state = tabs.get(active_tab)?;
    let model_key = select_model_key(tab_state, registry);
    let prompt_key = select_prompt_key(tab_state, prompt_registry);
    let system = resolve_system_prompt(tab_state, prompt_registry, args, &prompt_key);
    Some(QuestionTabSeed {
        category: tab_state.category.clone(),
        model_key,
        prompt_key,
        system,
        perf: args.perf,
        log_session_id: tab_state.app.log_session_id.clone(),
        prompts_dir: tab_state.app.prompts_dir.clone(),
        tavily_api_key: tab_state.app.tavily_api_key.clone(),
    })
}

fn select_model_key(
    tab_state: &TabState,
    registry: &crate::model_registry::ModelRegistry,
) -> String {
    if registry.get(&tab_state.app.model_key).is_some() {
        return tab_state.app.model_key.clone();
    }
    registry.default_key.clone()
}

fn select_prompt_key(
    tab_state: &TabState,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
) -> String {
    if prompt_registry.get(&tab_state.app.prompt_key).is_some() {
        return tab_state.app.prompt_key.clone();
    }
    prompt_registry.default_key.clone()
}

fn resolve_system_prompt(
    tab_state: &TabState,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    prompt_key: &str,
) -> String {
    tab_state
        .app
        .messages
        .iter()
        .find(|m| m.role == crate::types::ROLE_SYSTEM)
        .map(|m| m.content.clone())
        .or_else(|| {
            prompt_registry
                .get(prompt_key)
                .map(|p| p.content.clone())
        })
        .unwrap_or_else(|| args.system.clone())
}

fn create_question_tab(tabs: &mut Vec<TabState>, seed: &QuestionTabSeed) -> usize {
    let conv_id =
        crate::conversation::new_conversation_id().unwrap_or_else(|_| tabs.len().to_string());
    let mut tab = TabState::new(
        conv_id,
        seed.category.clone(),
        &seed.system,
        seed.perf,
        &seed.model_key,
        &seed.prompt_key,
    );
    tab.app.set_log_session_id(&seed.log_session_id);
    tab.app.prompts_dir = seed.prompts_dir.clone();
    tab.app.tavily_api_key = seed.tavily_api_key.clone();
    tab.app.model_key = seed.model_key.clone();
    tab.app.prompt_key = seed.prompt_key.clone();
    tabs.push(tab);
    tabs.len().saturating_sub(1)
}

fn ensure_category(
    categories: &mut Vec<String>,
    active_category: &mut usize,
    category: &str,
) {
    if !categories.iter().any(|c| c == category) {
        categories.push(category.to_string());
    }
    if let Some(idx) = categories.iter().position(|c| c == category) {
        *active_category = idx;
    }
}

fn start_question_request(
    tabs: &mut [TabState],
    tab_idx: usize,
    question: &str,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
    registry: &crate::model_registry::ModelRegistry,
) {
    let Some(params) = build_tab_request_params(tabs, tab_idx, question, args, tx, registry) else {
        return;
    };
    crate::ui::runtime_requests::start_tab_request(params);
}

fn build_tab_request_params<'a>(
    tabs: &'a mut [TabState],
    tab_idx: usize,
    question: &'a str,
    args: &'a Args,
    tx: &'a mpsc::Sender<RuntimeEvent>,
    registry: &'a crate::model_registry::ModelRegistry,
) -> Option<crate::ui::runtime_requests::StartTabRequestParams<'a>> {
    let tab_state = tabs.get_mut(tab_idx)?;
    let log_session_id = tab_state.app.log_session_id.clone();
    let model = model_for_tab(tab_state, registry);
    Some(tab_request_params(
        tab_state,
        question,
        model,
        args,
        tx,
        tab_idx,
        log_session_id,
    ))
}

fn model_for_tab<'a>(
    tab_state: &TabState,
    registry: &'a crate::model_registry::ModelRegistry,
) -> &'a crate::model_registry::ModelProfile {
    registry
        .get(&tab_state.app.model_key)
        .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"))
}

fn tab_request_params<'a>(
    tab_state: &'a mut TabState,
    question: &'a str,
    model: &'a crate::model_registry::ModelProfile,
    args: &'a Args,
    tx: &'a mpsc::Sender<RuntimeEvent>,
    tab_idx: usize,
    log_session_id: String,
) -> crate::ui::runtime_requests::StartTabRequestParams<'a> {
    crate::ui::runtime_requests::StartTabRequestParams {
        tab_state,
        question,
        base_url: &model.base_url,
        api_key: &model.api_key,
        model: &model.model,
        max_tokens: model.max_tokens,
        show_reasoning: args.show_reasoning,
        tx,
        tab_id: tab_idx,
        enable_web_search: args.web_search_enabled(),
        enable_code_exec: args.code_exec_enabled(),
        enable_read_file: args.read_file_enabled(),
        enable_read_code: args.read_code_enabled(),
        enable_modify_file: args.modify_file_enabled(),
        enable_ask_questions: args.ask_questions_enabled(),
        log_requests: args.log_requests.clone(),
        log_session_id,
    }
}

struct QuestionTabSeed {
    category: String,
    model_key: String,
    prompt_key: String,
    system: String,
    perf: bool,
    log_session_id: String,
    prompts_dir: String,
    tavily_api_key: String,
}
