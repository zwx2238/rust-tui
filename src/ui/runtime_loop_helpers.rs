use crate::args::Args;
use crate::session::SessionLocation;
use crate::types::Message;
use crate::ui::net::UiEvent;
use crate::ui::runtime_code_exec::{
    handle_code_exec_approve, handle_code_exec_deny, handle_code_exec_exit, handle_code_exec_stop,
};
use crate::ui::runtime_file_patch::{handle_file_patch_apply, handle_file_patch_cancel};
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::PendingCommand;

mod open_conversation;
mod category;

pub(crate) fn handle_pending_command(
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    pending: PendingCommand,
    session_location: &mut Option<SessionLocation>,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
) {
    if handle_session_command(pending, tabs, active_tab, categories, active_category, session_location) { return; }
    if handle_code_exec_command(pending, tabs, *active_tab, registry, args, tx) { return; }
    if handle_file_patch_command(pending, tabs, *active_tab, registry, args, tx) { return; }
    handle_tab_command(pending, tabs, active_tab, categories, active_category, registry, prompt_registry, args);
}

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

fn handle_session_command(
    pending: PendingCommand,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    session_location: &mut Option<SessionLocation>,
) -> bool {
    if let PendingCommand::SaveSession = pending {
        handle_save_session(tabs, active_tab, categories, active_category, session_location);
        return true;
    }
    false
}

fn handle_code_exec_command(
    pending: PendingCommand,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
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

fn handle_file_patch_command(
    pending: PendingCommand,
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
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

fn handle_tab_command(
    pending: PendingCommand,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    registry: &crate::model_registry::ModelRegistry,
    prompt_registry: &crate::llm::prompts::PromptRegistry,
    args: &Args,
) {
    match pending {
        PendingCommand::NewCategory => category::create_category_and_tab(
            tabs, active_tab, categories, active_category, registry, prompt_registry, args,
        ),
        PendingCommand::OpenConversation => open_conversation::open_conversation_in_tab(
            tabs, active_tab, categories, active_category, registry, prompt_registry, args,
        ),
        _ => {}
    }
}

fn handle_save_session(
    tabs: &mut Vec<TabState>,
    active_tab: &usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
    session_location: &mut Option<SessionLocation>,
) {
    save_all_conversations(tabs);
    let open_conversations = crate::ui::runtime_helpers::collect_open_conversations(tabs);
    let active_conv = tabs.get(*active_tab).map(|t| t.conversation_id.clone());
    let save_result = crate::session::save_session(
        categories,
        &open_conversations,
        active_conv.as_deref(),
        categories.get(*active_category).map(|s| s.as_str()),
        session_location.as_ref(),
    );
    if let Some(tab_state) = tabs.get_mut(*active_tab) {
        update_save_result(tab_state, save_result, session_location);
    }
}

fn save_all_conversations(tabs: &[TabState]) {
    for tab in tabs {
        let _ = crate::conversation::save_conversation(
            &crate::ui::runtime_helpers::tab_to_conversation(tab),
        );
    }
}

fn update_save_result(
    tab_state: &mut TabState,
    save_result: Result<SessionLocation, Box<dyn std::error::Error>>,
    session_location: &mut Option<SessionLocation>,
) {
    match save_result {
        Ok(loc) => {
            *session_location = Some(loc.clone());
            push_assistant_message(tab_state, format!("已保存会话：{}", loc.display_hint()));
        }
        Err(e) => {
            push_assistant_message(tab_state, format!("保存失败：{e}"));
        }
    }
}

fn push_assistant_message(tab_state: &mut TabState, content: String) {
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_ASSISTANT.to_string(),
        content,
        tool_call_id: None,
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
}

fn handle_code_exec_action(
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
    action: CodeExecAction,
) {
    let Some(tab_state) = tabs.get_mut(active_tab) else {
        return;
    };
    match action {
        CodeExecAction::Approve => handle_code_exec_approve(tab_state, active_tab, registry, args, tx),
        CodeExecAction::Deny => handle_code_exec_deny(tab_state, active_tab, registry, args, tx),
        CodeExecAction::Exit => handle_code_exec_exit(tab_state, active_tab, registry, args, tx),
        CodeExecAction::Stop => handle_code_exec_stop(tab_state),
    }
}

fn handle_file_patch_action(
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
    action: FilePatchAction,
) {
    let Some(tab_state) = tabs.get_mut(active_tab) else {
        return;
    };
    match action {
        FilePatchAction::Apply => handle_file_patch_apply(tab_state, active_tab, registry, args, tx),
        FilePatchAction::Cancel => {
            handle_file_patch_cancel(tab_state, active_tab, registry, args, tx)
        }
    }
}
