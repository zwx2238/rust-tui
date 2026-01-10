use super::{
    QuestionReviewSubmitParams,
    tabs::{QuestionTabSpawnParams, spawn_question_tabs},
};
use crate::args::Args;
use crate::types::Message;
use crate::ui::events::RuntimeEvent;
use crate::ui::notice::push_notice;
use crate::ui::runtime_helpers::TabState;
use crate::services::runtime_requests::start_followup_request;
use crate::ui::state::{PendingQuestionItem, PendingQuestionReview, QuestionDecision};
use std::sync::mpsc;

pub(crate) fn handle_question_review_submit(mut params: QuestionReviewSubmitParams<'_>) {
    let Some(tab_state) = params.tabs.get_mut(params.active_tab) else {
        return;
    };
    let Some(pending) = take_pending_review(tab_state) else {
        return;
    };
    let Some(pending) = ensure_review_ready(tab_state, pending) else {
        return;
    };
    let (approved, rejected, total) = split_questions(pending.questions);
    apply_review_submit(&mut params, approved, rejected, total, pending.call_id);
}

pub(crate) fn handle_question_review_cancel(
    tab_state: &mut TabState,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    let Some(pending) = tab_state.app.pending_question_review.take() else {
        return;
    };
    push_tool_message(
        &mut tab_state.app,
        r#"{"error":"用户取消"}"#.to_string(),
        pending.call_id,
    );
    start_followup(tab_state, registry, args, tx);
}

fn take_pending_review(tab_state: &mut TabState) -> Option<PendingQuestionReview> {
    tab_state.app.pending_question_review.take()
}

fn ensure_review_ready(
    tab_state: &mut TabState,
    pending: PendingQuestionReview,
) -> Option<PendingQuestionReview> {
    if has_pending_questions(&pending) {
        tab_state.app.pending_question_review = Some(pending);
        push_notice(&mut tab_state.app, "仍有未确认的问题");
        return None;
    }
    Some(pending)
}

fn has_pending_questions(pending: &PendingQuestionReview) -> bool {
    pending
        .questions
        .iter()
        .any(|q| q.decision == QuestionDecision::Pending)
}

fn split_questions(questions: Vec<PendingQuestionItem>) -> (Vec<ApprovedQuestion>, usize, usize) {
    let mut approved = Vec::new();
    let mut rejected = 0usize;
    let total = questions.len();
    for item in questions {
        match item.decision {
            QuestionDecision::Approved => approved.push(ApprovedQuestion {
                question: item.question,
                model_key: item.model_key,
            }),
            _ => rejected += 1,
        }
    }
    (approved, rejected, total)
}

fn apply_review_submit(
    params: &mut QuestionReviewSubmitParams<'_>,
    approved: Vec<ApprovedQuestion>,
    rejected: usize,
    total: usize,
    call_id: String,
) {
    spawn_question_tabs_if_needed(params, &approved);
    finalize_review_submit(params, approved.len(), rejected, total, call_id);
}

fn spawn_question_tabs_if_needed(
    params: &mut QuestionReviewSubmitParams<'_>,
    approved: &[ApprovedQuestion],
) {
    if approved.is_empty() {
        return;
    }
    spawn_question_tabs(
        QuestionTabSpawnParams {
            tabs: params.tabs,
            active_tab: params.active_tab,
            categories: params.categories,
            active_category: params.active_category,
            registry: params.registry,
            prompt_registry: params.prompt_registry,
            args: params.args,
            tx: params.tx,
        },
        approved,
    );
}

fn finalize_review_submit(
    params: &mut QuestionReviewSubmitParams<'_>,
    approved: usize,
    rejected: usize,
    total: usize,
    call_id: String,
) {
    let Some(tab_state) = params.tabs.get_mut(params.active_tab) else {
        return;
    };
    let content = build_submit_message(total, approved, rejected);
    push_tool_message(&mut tab_state.app, content, call_id);
    start_followup(
        tab_state,
        params.registry,
        params.args,
        params.tx,
    );
}

pub(super) struct ApprovedQuestion {
    pub(super) question: String,
    pub(super) model_key: String,
}

fn build_submit_message(total: usize, approved: usize, rejected: usize) -> String {
    format!(
        r#"{{"ok":true,"total":{total},"approved":{approved},"rejected":{rejected}}}"#,
    )
}

fn push_tool_message(app: &mut crate::ui::state::App, content: String, call_id: String) {
    let idx = app.messages.len();
    app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content,
        tool_call_id: Some(call_id),
        tool_calls: None,
    });
    app.dirty_indices.push(idx);
}

fn start_followup(
    tab_state: &mut TabState,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    let Some(params) = build_followup_params(tab_state, registry, args, tx) else {
        return;
    };
    start_followup_request(params);
}

fn build_followup_params<'a>(
    tab_state: &'a mut TabState,
    registry: &'a crate::model_registry::ModelRegistry,
    args: &'a Args,
    tx: &'a mpsc::Sender<RuntimeEvent>,
) -> Option<crate::services::runtime_requests::StartFollowupRequestParams<'a>> {
    let model = registry.get(&tab_state.app.model_key).unwrap_or_else(|| {
        registry.get(&registry.default_key).expect("model")
    });
    let log_session_id = tab_state.app.log_session_id.clone();
    Some(crate::services::runtime_requests::StartFollowupRequestParams {
        tab_state,
        base_url: &model.base_url,
        api_key: &model.api_key,
        model: &model.model,
        max_tokens: model.max_tokens,
        show_reasoning: args.show_reasoning,
        tx,
        enable_web_search: args.web_search_enabled(),
        enable_code_exec: args.code_exec_enabled(),
        enable_read_file: args.read_file_enabled(),
        enable_read_code: args.read_code_enabled(),
        enable_modify_file: args.modify_file_enabled(),
        enable_ask_questions: args.ask_questions_enabled(),
        log_requests: args.log_requests.clone(),
        log_session_id,
    })
}
