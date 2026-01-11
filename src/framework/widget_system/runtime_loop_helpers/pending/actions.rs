use crate::args::Args;
use crate::framework::widget_system::runtime::events::RuntimeEvent;
use crate::services::runtime_code_exec::{
    handle_code_exec_approve, handle_code_exec_deny, handle_code_exec_exit, handle_code_exec_stop,
};
use crate::services::runtime_file_patch::{handle_file_patch_apply, handle_file_patch_cancel};
use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::services::runtime_question_review::{
    QuestionReviewSubmitParams, handle_question_review_cancel, handle_question_review_submit,
};
use crate::framework::widget_system::runtime::state::PendingCommand;

enum CodeExecAction {
    Approve,
    Deny,
    Exit,
    Stop,
}

enum FilePatchAction {
    Apply,
    Cancel,
}

enum QuestionReviewAction {
    Submit,
    Cancel,
}

pub(crate) struct QuestionReviewCommandParams<'a> {
    pub pending: PendingCommand,
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
    pub tx: &'a std::sync::mpsc::Sender<RuntimeEvent>,
}

pub(crate) fn handle_code_exec_command(
    pending: PendingCommand,
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
) -> bool {
    let action = match pending {
        PendingCommand::ApproveCodeExec => Some(CodeExecAction::Approve),
        PendingCommand::DenyCodeExec => Some(CodeExecAction::Deny),
        PendingCommand::ExitCodeExec => Some(CodeExecAction::Exit),
        PendingCommand::StopCodeExec => Some(CodeExecAction::Stop),
        _ => None,
    };
    if let Some(action) = action {
        handle_code_exec_action(tabs, active_tab, registry, args, tx, action);
        return true;
    }
    false
}

pub(crate) fn handle_file_patch_command(
    pending: PendingCommand,
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
) -> bool {
    let action = match pending {
        PendingCommand::ApplyFilePatch => Some(FilePatchAction::Apply),
        PendingCommand::CancelFilePatch => Some(FilePatchAction::Cancel),
        _ => None,
    };
    if let Some(action) = action {
        handle_file_patch_action(tabs, active_tab, registry, args, tx, action);
        return true;
    }
    false
}

pub(crate) fn handle_question_review_command(params: QuestionReviewCommandParams<'_>) -> bool {
    let action = match params.pending {
        PendingCommand::SubmitQuestionReview => Some(QuestionReviewAction::Submit),
        PendingCommand::CancelQuestionReview => Some(QuestionReviewAction::Cancel),
        _ => None,
    };
    let Some(action) = action else {
        return false;
    };
    handle_question_review_action(params, action);
    true
}

fn handle_code_exec_action(
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
    action: CodeExecAction,
) {
    let Some(tab_state) = tabs.get_mut(active_tab) else {
        return;
    };
    match action {
        CodeExecAction::Approve => {
            handle_code_exec_approve(tab_state, active_tab, registry, args, tx)
        }
        CodeExecAction::Deny => handle_code_exec_deny(tab_state, registry, args, tx),
        CodeExecAction::Exit => handle_code_exec_exit(tab_state, registry, args, tx),
        CodeExecAction::Stop => handle_code_exec_stop(tab_state),
    }
}

fn handle_file_patch_action(
    tabs: &mut [TabState],
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<RuntimeEvent>,
    action: FilePatchAction,
) {
    let Some(tab_state) = tabs.get_mut(active_tab) else {
        return;
    };
    match action {
        FilePatchAction::Apply => handle_file_patch_apply(tab_state, registry, args, tx),
        FilePatchAction::Cancel => handle_file_patch_cancel(tab_state, registry, args, tx),
    }
}

fn handle_question_review_action(
    params: QuestionReviewCommandParams<'_>,
    action: QuestionReviewAction,
) {
    match action {
        QuestionReviewAction::Submit => handle_question_review_submit(QuestionReviewSubmitParams {
            tabs: params.tabs,
            active_tab: params.active_tab,
            categories: params.categories,
            active_category: params.active_category,
            registry: params.registry,
            prompt_registry: params.prompt_registry,
            args: params.args,
            tx: params.tx,
        }),
        QuestionReviewAction::Cancel => {
            if let Some(tab_state) = params.tabs.get_mut(params.active_tab) {
                handle_question_review_cancel(
                    tab_state,
                    params.registry,
                    params.args,
                    params.tx,
                );
            }
        }
    }
}
