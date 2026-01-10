use crate::args::Args;
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_helpers::TabState;
use std::sync::mpsc;

mod submit;
mod tabs;
mod tool;

pub(crate) struct QuestionReviewSubmitParams<'a> {
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
    pub tx: &'a mpsc::Sender<RuntimeEvent>,
}

pub(crate) use submit::{handle_question_review_cancel, handle_question_review_submit};
pub(crate) use tool::{
    all_questions_decided, cycle_question_model, cycle_question_model_prev,
    handle_question_review_request, selected_model_key, set_all_decisions, set_all_models,
    set_question_decision, toggle_question_decision,
};
