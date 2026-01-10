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
) -> Result<(), String> {
    if tab_state.app.pending_question_review.is_some() {
        return Err("已有待审批的问题集".to_string());
    }
    let args: QuestionReviewArgs = serde_json::from_str(&call.function.arguments)
        .map_err(|e| format!("ask_questions 参数解析失败：{e}"))?;
    let questions = sanitize_questions(args.questions)?;
    tab_state.app.pending_question_review = Some(PendingQuestionReview {
        call_id: call.id.clone(),
        questions: questions
            .into_iter()
            .map(|q| PendingQuestionItem {
                question: q,
                decision: QuestionDecision::Pending,
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
