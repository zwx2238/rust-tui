use crate::types::{Message, ToolCall};
use crate::ui::runtime_helpers::TabState;

pub(super) struct ToolApplyState {
    pub(super) any_results: bool,
    pub(super) needs_approval: bool,
    pub(super) api_key: String,
}

impl ToolApplyState {
    pub(super) fn new(api_key: String) -> Self {
        Self {
            any_results: false,
            needs_approval: false,
            api_key,
        }
    }
}

pub(super) enum ToolKind {
    WebSearch,
    ReadFile,
    ReadCode,
    ListDir,
}

pub(super) fn push_tool_disabled(
    tab_state: &mut TabState,
    call: &ToolCall,
    state: &mut ToolApplyState,
) {
    let msg = format!(r#"{{"error":"{} 未启用"}}"#, call.function.name);
    push_tool_message(tab_state, call, msg);
    state.any_results = true;
}

pub(super) fn push_workspace_error(
    tab_state: &mut TabState,
    call: &ToolCall,
    state: &mut ToolApplyState,
    err: &str,
) {
    push_tool_message(tab_state, call, format!(r#"{{"error":"{}"}}"#, err));
    state.any_results = true;
}

pub(super) fn push_tool_message(tab_state: &mut TabState, call: &ToolCall, content: String) {
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content,
        tool_call_id: Some(call.id.clone()),
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
}

pub(super) fn push_tool_error(
    tab_state: &mut TabState,
    call: &ToolCall,
    state: &mut ToolApplyState,
    message: impl AsRef<str>,
) {
    push_tool_message(
        tab_state,
        call,
        format!(r#"{{"error":"{}"}}"#, message.as_ref()),
    );
    state.any_results = true;
}

pub(super) fn push_assistant_message(tab_state: &mut TabState, content: String) {
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_ASSISTANT.to_string(),
        content,
        tool_call_id: None,
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
}
