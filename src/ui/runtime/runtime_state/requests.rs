use super::RunState;
use crate::args::Args;
use crate::model_registry::ModelRegistry;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_tab_request;
use std::sync::mpsc;

pub(crate) fn run_initial_requests(
    question_set: Option<&Vec<String>>,
    auto_retry: Option<(usize, String)>,
    state: &mut RunState,
    registry: &ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
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
    tx: &mpsc::Sender<UiEvent>,
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
    tx: &mpsc::Sender<UiEvent>,
) {
    start_tab_request_for_question(tab_idx, question, state, registry, args, tx);
}

fn start_tab_request_for_question(
    tab_idx: usize,
    question: &str,
    state: &mut RunState,
    registry: &ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some(tab_state) = state.tabs.get_mut(tab_idx) else {
        return;
    };
    let model = model_for_tab(tab_state, registry);
    start_tab_request(
        tab_state,
        question,
        &model.base_url,
        &model.api_key,
        &model.model,
        args.show_reasoning,
        tx,
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

fn model_for_tab<'a>(tab_state: &TabState, registry: &'a ModelRegistry) -> &'a crate::model_registry::ModelProfile {
    registry
        .get(&tab_state.app.model_key)
        .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"))
}
