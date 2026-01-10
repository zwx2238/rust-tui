use super::RunState;
use crate::args::Args;
use crate::model_registry::ModelRegistry;
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_helpers::TabState;
use crate::services::runtime_requests::start_tab_request;
use std::sync::mpsc;

pub(crate) fn run_initial_requests(
    question_set: Option<&Vec<String>>,
    auto_retry: Option<(usize, String)>,
    state: &mut RunState,
    registry: &ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    if let Some(questions) = question_set {
        start_question_set_requests(questions, state, registry, args, tx);
    }
    if let Some((tab_idx, question)) = auto_retry {
        start_retry_request(tab_idx, &question, state, registry, args, tx);
    }
}

fn start_question_set_requests(
    questions: &[String],
    state: &mut RunState,
    registry: &ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    for (i, question) in questions.iter().enumerate() {
        start_tab_request_for_question(i, question, state, registry, args, tx);
    }
}

fn start_retry_request(
    tab_idx: usize,
    question: &str,
    state: &mut RunState,
    registry: &ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    start_tab_request_for_question(tab_idx, question, state, registry, args, tx);
}

fn start_tab_request_for_question(
    tab_idx: usize,
    question: &str,
    state: &mut RunState,
    registry: &ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    let Some(tab_state) = state.tabs.get_mut(tab_idx) else {
        return;
    };
    let model = model_for_tab(tab_state, registry);
    let flags = request_flags(args);
    let params = build_request_params(tab_state, question, model, args, tx, flags);
    start_tab_request(params);
}

fn model_for_tab<'a>(
    tab_state: &TabState,
    registry: &'a ModelRegistry,
) -> &'a crate::model_registry::ModelProfile {
    registry
        .get(&tab_state.app.model_key)
        .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"))
}

struct RequestFlags {
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    enable_ask_questions: bool,
    log_requests: Option<String>,
}

fn request_flags(args: &Args) -> RequestFlags {
    RequestFlags {
        enable_web_search: args.web_search_enabled(),
        enable_code_exec: args.code_exec_enabled(),
        enable_read_file: args.read_file_enabled(),
        enable_read_code: args.read_code_enabled(),
        enable_modify_file: args.modify_file_enabled(),
        enable_ask_questions: args.ask_questions_enabled(),
        log_requests: args.log_requests.clone(),
    }
}

fn build_request_params<'a>(
    tab_state: &'a mut TabState,
    question: &'a str,
    model: &'a crate::model_registry::ModelProfile,
    args: &'a Args,
    tx: &'a mpsc::Sender<RuntimeEvent>,
    flags: RequestFlags,
) -> crate::services::runtime_requests::StartTabRequestParams<'a> {
    let log_session_id = tab_state.app.log_session_id.clone();
    crate::services::runtime_requests::StartTabRequestParams {
        tab_state,
        question,
        base_url: &model.base_url,
        api_key: &model.api_key,
        model: &model.model,
        max_tokens: model.max_tokens,
        show_reasoning: args.show_reasoning,
        tx,
        enable_web_search: flags.enable_web_search,
        enable_code_exec: flags.enable_code_exec,
        enable_read_file: flags.enable_read_file,
        enable_read_code: flags.enable_read_code,
        enable_modify_file: flags.enable_modify_file,
        enable_ask_questions: flags.enable_ask_questions,
        log_requests: flags.log_requests,
        log_session_id,
    }
}
