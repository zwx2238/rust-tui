use crate::args::Args;
use crate::model_registry::ModelRegistry;
use crate::types::{Message, ToolCall};
use crate::ui::net::UiEvent;
use crate::ui::runtime_code_exec::handle_code_exec_request;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_followup_request;
use crate::ui::tools::run_tool;
use std::sync::mpsc;

pub struct ToolService<'a> {
    registry: &'a ModelRegistry,
    args: &'a Args,
    tx: &'a mpsc::Sender<UiEvent>,
}

impl<'a> ToolService<'a> {
    pub fn new(
        registry: &'a ModelRegistry,
        args: &'a Args,
        tx: &'a mpsc::Sender<UiEvent>,
    ) -> Self {
        Self { registry, args, tx }
    }

    pub fn apply_tool_calls(
        &self,
        tab_state: &mut TabState,
        tab_id: usize,
        calls: &[ToolCall],
    ) {
        let mut any_results = false;
        let mut needs_approval = false;
        let api_key = tab_state.app.tavily_api_key.clone();
        for call in calls {
            if call.function.name == "web_search" {
                if !self.args.web_search_enabled() {
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
                let result = run_tool(call, &api_key);
                let idx = tab_state.app.messages.len();
                tab_state.app.messages.push(Message {
                    role: crate::types::ROLE_TOOL.to_string(),
                    content: result.content,
                    tool_call_id: Some(call.id.clone()),
                    tool_calls: None,
                });
                tab_state.app.dirty_indices.push(idx);
                if result.has_results {
                    any_results = true;
                }
                continue;
            }
            if call.function.name == "read_file" || call.function.name == "read_code" {
                let enabled = if call.function.name == "read_file" {
                    self.args.read_file_enabled()
                } else {
                    self.args.read_code_enabled()
                };
                if !enabled {
                    let idx = tab_state.app.messages.len();
                    tab_state.app.messages.push(Message {
                        role: crate::types::ROLE_TOOL.to_string(),
                        content: format!(r#"{{"error":"{} 未启用"}}"#, call.function.name),
                        tool_call_id: Some(call.id.clone()),
                        tool_calls: None,
                    });
                    tab_state.app.dirty_indices.push(idx);
                    any_results = true;
                    continue;
                }
                let result = run_tool(call, &api_key);
                let idx = tab_state.app.messages.len();
                tab_state.app.messages.push(Message {
                    role: crate::types::ROLE_TOOL.to_string(),
                    content: result.content,
                    tool_call_id: Some(call.id.clone()),
                    tool_calls: None,
                });
                tab_state.app.dirty_indices.push(idx);
                if result.has_results {
                    any_results = true;
                }
                continue;
            }
            if call.function.name == "code_exec" {
                if !self.args.code_exec_enabled() {
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
        let model = self
            .registry
            .get(&tab_state.app.model_key)
            .unwrap_or_else(|| self.registry.get(&self.registry.default_key).expect("model"));
        start_followup_request(
            tab_state,
            &model.base_url,
            &model.api_key,
            &model.model,
            self.args.show_reasoning,
            self.tx,
            tab_id,
            self.args.web_search_enabled(),
            self.args.code_exec_enabled(),
            self.args.read_file_enabled(),
            self.args.read_code_enabled(),
            self.args.log_requests.clone(),
            tab_state.app.log_session_id.clone(),
        );
    }
}
