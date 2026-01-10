mod actions;
mod session;
mod tab;

use crate::args::Args;
use crate::session::SessionLocation;
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::PendingCommand;

pub(crate) struct HandlePendingCommandParams<'a> {
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: &'a mut usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub pending: PendingCommand,
    pub session_location: &'a mut Option<SessionLocation>,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
    pub tx: &'a std::sync::mpsc::Sender<RuntimeEvent>,
}

pub(crate) struct HandlePendingCommandIfAnyParams<'a> {
    pub pending_command: Option<PendingCommand>,
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: &'a mut usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub session_location: &'a mut Option<SessionLocation>,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
    pub tx: &'a std::sync::mpsc::Sender<RuntimeEvent>,
}

pub(crate) fn handle_pending_command(params: HandlePendingCommandParams<'_>) {
    let mut params = params;
    if handled_by_actions(&mut params) {
        return;
    }
    tab::handle_tab_command(tab_params(&mut params));
}

pub(crate) fn handle_pending_command_if_any(params: HandlePendingCommandIfAnyParams<'_>) {
    if let Some(pending) = params.pending_command {
        handle_pending_command(HandlePendingCommandParams {
            pending,
            tabs: params.tabs,
            active_tab: params.active_tab,
            categories: params.categories,
            active_category: params.active_category,
            session_location: params.session_location,
            registry: params.registry,
            prompt_registry: params.prompt_registry,
            args: params.args,
            tx: params.tx,
        });
    }
}

fn handled_by_actions(params: &mut HandlePendingCommandParams<'_>) -> bool {
    if session::handle_session_command(
        params.pending,
        params.tabs,
        params.active_tab,
        params.categories,
        params.active_category,
        params.session_location,
    ) {
        return true;
    }
    if actions::handle_code_exec_command(
        params.pending,
        params.tabs,
        *params.active_tab,
        params.registry,
        params.args,
        params.tx,
    ) {
        return true;
    }
    if actions::handle_file_patch_command(
        params.pending,
        params.tabs,
        *params.active_tab,
        params.registry,
        params.args,
        params.tx,
    ) {
        return true;
    }
    actions::handle_question_review_command(actions::QuestionReviewCommandParams {
        pending: params.pending,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        tx: params.tx,
    })
}

fn tab_params<'a>(
    params: &'a mut HandlePendingCommandParams<'a>,
) -> tab::HandleTabCommandParams<'a> {
    tab::HandleTabCommandParams {
        pending: params.pending,
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
    }
}
