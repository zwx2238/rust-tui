use crate::args::Args;
use crate::session::SessionLocation;
use crate::ui::net::UiEvent;
use crate::ui::runtime_code_exec::{
    handle_code_exec_approve, handle_code_exec_deny, handle_code_exec_exit,
    handle_code_exec_request, handle_code_exec_stop,
};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_followup_request;
use crate::ui::state::PendingCommand;
use crate::ui::tools::{ToolResult, run_tool};
use crate::types::Message;
use std::sync::mpsc;

pub(crate) fn apply_tool_calls(
    tab_state: &mut TabState,
    tab_id: usize,
    calls: &[crate::types::ToolCall],
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let mut any_results = false;
    let mut needs_approval = false;
    let api_key = tab_state.app.tavily_api_key.clone();
    for call in calls {
        if call.function.name == "web_search" {
            if !args.enable_web_search {
                let idx = tab_state.app.messages.len();
                tab_state.app.messages.push(Message {
                    role: crate::types::ROLE_TOOL.to_string(),
                    content: r#"{"error":"web_search 未启用"}"#.to_string(),
                    tool_call_id: Some(call.id.clone()),
                    tool_calls: None,
                });
                tab_state.app.dirty_indices.push(idx);
                any_results = true;
                continue;
            }
            let ToolResult {
                content,
                has_results,
            } = run_tool(call, &api_key);
            let idx = tab_state.app.messages.len();
            tab_state.app.messages.push(Message {
                role: crate::types::ROLE_TOOL.to_string(),
                content,
                tool_call_id: Some(call.id.clone()),
                tool_calls: None,
            });
            tab_state.app.dirty_indices.push(idx);
            if has_results {
                any_results = true;
            }
            continue;
        }
        if call.function.name == "code_exec" {
            if !args.enable_code_exec {
                let idx = tab_state.app.messages.len();
                tab_state.app.messages.push(Message {
                    role: crate::types::ROLE_TOOL.to_string(),
                    content: r#"{"error":"code_exec 未启用"}"#.to_string(),
                    tool_call_id: Some(call.id.clone()),
                    tool_calls: None,
                });
                tab_state.app.dirty_indices.push(idx);
                any_results = true;
                continue;
            }
            match handle_code_exec_request(tab_state, call) {
                Ok(()) => {
                    needs_approval = true;
                    any_results = true;
                }
                Err(err) => {
                    let idx = tab_state.app.messages.len();
                    tab_state.app.messages.push(Message {
                        role: crate::types::ROLE_TOOL.to_string(),
                        content: format!(r#"{{"error":"{err}"}}"#),
                        tool_call_id: Some(call.id.clone()),
                        tool_calls: None,
                    });
                    tab_state.app.dirty_indices.push(idx);
                    any_results = true;
                }
            }
        }
    }
    if needs_approval {
        return;
    }
    if !any_results {
        let idx = tab_state.app.messages.len();
        tab_state.app.messages.push(Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "未找到可靠结果，无法确认。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab_state.app.dirty_indices.push(idx);
        return;
    }
    let model = registry
        .get(&tab_state.app.model_key)
        .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"));
    start_followup_request(
        tab_state,
        &model.base_url,
        &model.api_key,
        &model.model,
        args.show_reasoning,
        tx,
        tab_id,
        args.enable_web_search,
        args.enable_code_exec,
        args.log_requests.clone(),
    );
}

pub(crate) fn handle_pending_command(
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    pending: PendingCommand,
    session_location: &mut Option<SessionLocation>,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
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
