use crate::args::Args;
use crate::session::SessionLocation;
use crate::ui::runtime_code_exec::{
    handle_code_exec_approve, handle_code_exec_deny, handle_code_exec_exit,
    handle_code_exec_stop,
};
use crate::ui::runtime_helpers::TabState;
use crate::ui::net::UiEvent;
use crate::ui::state::PendingCommand;
use crate::types::Message;

pub(crate) fn handle_pending_command(
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    pending: PendingCommand,
    session_location: &mut Option<SessionLocation>,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &std::sync::mpsc::Sender<UiEvent>,
) {
    match pending {
        PendingCommand::SaveSession => {
            let snapshot = crate::ui::runtime_helpers::collect_session_tabs(tabs);
            let save_result = crate::session::save_session(
                &snapshot,
                active_tab,
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
    }
}
