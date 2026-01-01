use crate::args::Args;
use crate::session::SessionLocation;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{TabState, start_followup_request};
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
    let api_key = tab_state.app.tavily_api_key.clone();
    for call in calls {
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

pub(crate) fn handle_pending_command(
    tabs: &mut Vec<TabState>,
    active_tab: usize,
    pending: PendingCommand,
    session_location: &mut Option<SessionLocation>,
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
    }
}
