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

pub(crate) fn handle_pending_command(
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    categories: &[String],
    active_category: usize,
    pending: PendingCommand,
    session_location: &mut Option<SessionLocation>,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
) {
    match pending {
        PendingCommand::SaveSession => {
            for tab in &*tabs {
                let _ = crate::conversation::save_conversation(
                    &crate::ui::runtime_helpers::tab_to_conversation(tab),
                );
            }
            let open_conversations = crate::ui::runtime_helpers::collect_open_conversations(tabs);
            let active_conv = tabs.get(active_tab).map(|t| t.conversation_id.clone());
            let save_result = crate::session::save_session(
                categories,
                &open_conversations,
                active_conv.as_deref(),
                categories.get(active_category).map(|s| s.as_str()),
                session_location.as_ref(),
            );
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                match save_result {
                    Ok(loc) => {
                        *session_location = Some(loc.clone());
                        let hint = loc.display_hint();
                        let idx = tab_state.app.messages.len();
                        tab_state.app.messages.push(Message {
                            role: crate::types::ROLE_ASSISTANT.to_string(),
                            content: format!("已保存会话：{hint}"),
                            tool_call_id: None,
                            tool_calls: None,
                        });
                        tab_state.app.dirty_indices.push(idx);
                    }
                    Err(e) => {
                        let idx = tab_state.app.messages.len();
                        tab_state.app.messages.push(Message {
                            role: crate::types::ROLE_ASSISTANT.to_string(),
                            content: format!("保存失败：{e}"),
                            tool_call_id: None,
                            tool_calls: None,
                        });
                        tab_state.app.dirty_indices.push(idx);
                    }
                }
            }
        }
        PendingCommand::ApproveCodeExec => {
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                handle_code_exec_approve(tab_state, active_tab, registry, args, tx);
            }
        }
        PendingCommand::DenyCodeExec => {
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                handle_code_exec_deny(tab_state, active_tab, registry, args, tx);
            }
        }
        PendingCommand::ExitCodeExec => {
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                handle_code_exec_exit(tab_state, active_tab, registry, args, tx);
            }
        }
        PendingCommand::StopCodeExec => {
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                handle_code_exec_stop(tab_state);
            }
        }
        PendingCommand::ApplyFilePatch => {
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                handle_file_patch_apply(tab_state, active_tab, registry, args, tx);
            }
        }
        PendingCommand::CancelFilePatch => {
            if let Some(tab_state) = tabs.get_mut(active_tab) {
                handle_file_patch_cancel(tab_state, active_tab, registry, args, tx);
            }
        }
    }
}
