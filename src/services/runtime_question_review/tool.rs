use crate::ui::runtime_helpers::TabState;
use crate::ui::state::{PendingQuestionItem, PendingQuestionReview, QuestionDecision};
use serde::Deserialize;

#[derive(Deserialize)]
struct QuestionReviewArgs {
    questions: Vec<String>,
}

pub(crate) fn handle_question_review_request(
    tab_state: &mut TabState,
    call: &crate::types::ToolCall,
    registry: &crate::model_registry::ModelRegistry,
) -> Result<(), String> {
    if tab_state.app.pending_question_review.is_some() {
        return Err("已有待审批的问题集".to_string());
    }
    let args: QuestionReviewArgs = serde_json::from_str(&call.function.arguments)
        .map_err(|e| format!("ask_questions 参数解析失败：{e}"))?;
    let questions = sanitize_questions(args.questions)?;
    ensure_questions_independent(&questions)?;
    let default_model = resolve_default_model(tab_state, registry);
    tab_state.app.pending_question_review = Some(PendingQuestionReview {
        call_id: call.id.clone(),
        questions: questions
            .into_iter()
            .map(|q| PendingQuestionItem {
                question: q,
                decision: QuestionDecision::Pending,
                model_key: default_model.clone(),
            })
            .collect(),
    });
    Ok(())
}

pub(crate) fn toggle_question_decision(tab_state: &mut TabState, idx: usize) -> bool {
    let Some(item) = pending_item_mut(tab_state, idx) else {
        return false;
    };
    item.decision = match item.decision {
        QuestionDecision::Pending => QuestionDecision::Approved,
        QuestionDecision::Approved => QuestionDecision::Rejected,
        QuestionDecision::Rejected => QuestionDecision::Approved,
    };
    true
}

pub(crate) fn set_question_decision(
    tab_state: &mut TabState,
    idx: usize,
    decision: QuestionDecision,
) -> bool {
    let Some(item) = pending_item_mut(tab_state, idx) else {
        return false;
    };
    item.decision = decision;
    true
}

pub(crate) fn set_all_decisions(
    tab_state: &mut TabState,
    decision: QuestionDecision,
) -> bool {
    let Some(pending) = tab_state.app.pending_question_review.as_mut() else {
        return false;
    };
    for item in &mut pending.questions {
        item.decision = decision;
    }
    true
}

pub(crate) fn cycle_question_model(
    tab_state: &mut TabState,
    idx: usize,
    registry: &crate::model_registry::ModelRegistry,
) -> bool {
    let Some(item) = pending_item_mut(tab_state, idx) else {
        return false;
    };
    let Some(next) = next_model_key(&item.model_key, registry) else {
        return false;
    };
    item.model_key = next;
    true
}

pub(crate) fn cycle_question_model_prev(
    tab_state: &mut TabState,
    idx: usize,
    registry: &crate::model_registry::ModelRegistry,
) -> bool {
    let Some(item) = pending_item_mut(tab_state, idx) else {
        return false;
    };
    let Some(prev) = prev_model_key(&item.model_key, registry) else {
        return false;
    };
    item.model_key = prev;
    true
}

pub(crate) fn set_all_models(
    tab_state: &mut TabState,
    model_key: &str,
) -> bool {
    let Some(pending) = tab_state.app.pending_question_review.as_mut() else {
        return false;
    };
    for item in &mut pending.questions {
        item.model_key = model_key.to_string();
    }
    true
}

pub(crate) fn selected_model_key(tab_state: &TabState, idx: usize) -> Option<&str> {
    tab_state
        .app
        .pending_question_review
        .as_ref()
        .and_then(|pending| pending.questions.get(idx))
        .map(|item| item.model_key.as_str())
}

pub(crate) fn all_questions_decided(tab_state: &TabState) -> bool {
    tab_state
        .app
        .pending_question_review
        .as_ref()
        .map(|pending| {
            pending
                .questions
                .iter()
                .all(|q| q.decision != QuestionDecision::Pending)
        })
        .unwrap_or(true)
}

fn pending_item_mut(tab_state: &mut TabState, idx: usize) -> Option<&mut PendingQuestionItem> {
    tab_state
        .app
        .pending_question_review
        .as_mut()
        .and_then(|pending| pending.questions.get_mut(idx))
}

fn sanitize_questions(raw: Vec<String>) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for q in raw {
        let trimmed = q.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
    }
    if out.is_empty() {
        return Err("ask_questions 至少需要一个问题".to_string());
    }
    Ok(out)
}

fn ensure_questions_independent(questions: &[String]) -> Result<(), String> {
    for question in questions {
        if has_dependency_marker(question) {
            return Err("ask_questions 的每个问题必须自洽，不能依赖其他问题或上文".to_string());
        }
    }
    Ok(())
}

fn has_dependency_marker(text: &str) -> bool {
    let lower = text.to_lowercase();
    let markers = [
        "上文",
        "前文",
        "上述",
        "以上",
        "如下",
        "如上",
        "同上",
        "见上",
        "上一题",
        "上一个问题",
        "the above",
        "previous question",
        "earlier question",
        "as mentioned above",
    ];
    markers.iter().any(|marker| lower.contains(marker))
}

fn resolve_default_model(
    tab_state: &TabState,
    registry: &crate::model_registry::ModelRegistry,
) -> String {
    if registry.get(&tab_state.app.model_key).is_some() {
        return tab_state.app.model_key.clone();
    }
    registry.default_key.clone()
}

fn next_model_key(
    current: &str,
    registry: &crate::model_registry::ModelRegistry,
) -> Option<String> {
    if registry.models.is_empty() {
        return None;
    }
    let idx = registry.index_of(current).unwrap_or(0);
    let next = (idx + 1) % registry.models.len();
    registry.models.get(next).map(|m| m.key.clone())
}

fn prev_model_key(
    current: &str,
    registry: &crate::model_registry::ModelRegistry,
) -> Option<String> {
    if registry.models.is_empty() {
        return None;
    }
    let idx = registry.index_of(current).unwrap_or(0);
    let prev = idx.wrapping_add(registry.models.len() - 1) % registry.models.len();
    registry.models.get(prev).map(|m| m.key.clone())
}
