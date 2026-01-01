use crate::args::Args;
use crate::session::SessionLocation;
use crate::ui::code_exec::run_python_in_docker;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{TabState, start_followup_request};
use crate::ui::state::{PendingCodeExec, PendingCommand};
use crate::ui::tools::{CodeExecRequest, ToolResult, parse_code_exec_args, run_tool};
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
    );
}

fn handle_code_exec_request(
    tab_state: &mut TabState,
    call: &crate::types::ToolCall,
) -> Result<(), String> {
    if tab_state.app.pending_code_exec.is_some() {
        return Err("已有待审批的代码执行请求".to_string());
    }
    let CodeExecRequest { language, code } = parse_code_exec_args(&call.function.arguments)?;
    tab_state.app.pending_code_exec = Some(PendingCodeExec {
        call_id: call.id.clone(),
        language,
        code,
    });
    tab_state.app.code_exec_selection = 0;
    Ok(())
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
    }
}

fn handle_code_exec_approve(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some(pending) = tab_state.app.pending_code_exec.take() else {
        let idx = tab_state.app.messages.len();
        tab_state.app.messages.push(Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "没有待审批的代码执行请求。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab_state.app.dirty_indices.push(idx);
        return;
    };
    let output = match pending.language.as_str() {
        "python" => run_python_in_docker(&pending.code)
            .map(|out| {
                serde_json::json!({
                    "language": pending.language,
                    "exit_code": out.exit_code,
                    "stdout": out.stdout,
                    "stderr": out.stderr
                })
                .to_string()
            })
            .unwrap_or_else(|e| format!(r#"{{"error":"{e}"}}"#)),
        _ => format!(r#"{{"error":"不支持的语言：{}"}}"#, pending.language),
    };
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content: output,
        tool_call_id: Some(pending.call_id),
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
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
    );
}

fn handle_code_exec_deny(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some(pending) = tab_state.app.pending_code_exec.take() else {
        let idx = tab_state.app.messages.len();
        tab_state.app.messages.push(Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "没有待审批的代码执行请求。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab_state.app.dirty_indices.push(idx);
        return;
    };
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content: r#"{"error":"用户拒绝执行"}"#.to_string(),
        tool_call_id: Some(pending.call_id),
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
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
    );
}
