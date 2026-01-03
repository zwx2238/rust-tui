use crate::types::{ToolCall, ToolFunctionCall, Usage};
use rig::completion::GetTokenUsage;
use std::sync::mpsc::Sender;

use super::net_logging::write_response_log;
use super::types::{LlmEvent, UiEvent};

pub(super) fn log_request(
    input: &super::request::RequestInput,
    ctx: &crate::llm::rig::RigRequestContext,
) {
    if let Some(dir) = input.log_dir.as_deref() {
        let _ = super::net_logging::write_request_log(
            dir,
            &input.log_session_id,
            input.tab,
            input.message_index,
            &input.base_url,
            &input.model,
            ctx,
        );
    }
}

pub(super) fn log_response_text(input: &super::request::RequestInput, text: &str) {
    if let Some(dir) = input.log_dir.as_deref() {
        let _ = write_response_log(
            dir,
            &input.log_session_id,
            input.tab,
            input.message_index,
            text,
        );
    }
}

pub(super) fn send_chunk(tx: &Sender<UiEvent>, input: &super::request::RequestInput, chunk: String) {
    let _ = tx.send(UiEvent {
        tab: input.tab,
        request_id: input.request_id,
        event: LlmEvent::Chunk(chunk),
    });
}

pub(super) fn send_done(
    input: &super::request::RequestInput,
    tx: &Sender<UiEvent>,
    usage: Option<Usage>,
) {
    let _ = tx.send(UiEvent {
        tab: input.tab,
        request_id: input.request_id,
        event: LlmEvent::Done { usage },
    });
}

pub(super) fn send_tool_calls(
    input: &super::request::RequestInput,
    tx: &Sender<UiEvent>,
    calls: Vec<ToolCall>,
    usage: Option<Usage>,
) {
    let _ = tx.send(UiEvent {
        tab: input.tab,
        request_id: input.request_id,
        event: LlmEvent::ToolCalls { calls, usage },
    });
}

pub(super) fn convert_tool_call(
    call: &rig::message::ToolCall,
    tab: usize,
    request_id: u64,
) -> ToolCall {
    ToolCall {
        id: format!("rig-{}-{}", tab, request_id),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: call.function.name.clone(),
            arguments: serde_json::to_string(&call.function.arguments).unwrap_or_default(),
        },
    }
}

pub(super) fn map_usage(usage: rig::completion::Usage) -> Usage {
    Usage {
        prompt_tokens: Some(usage.input_tokens),
        completion_tokens: Some(usage.output_tokens),
        total_tokens: Some(usage.total_tokens),
    }
}

pub(super) fn usage_from_stream(res: &super::stream::OpenAiStreamResponse) -> Option<Usage> {
    res.token_usage().map(map_usage)
}

pub(super) fn log_tool_call(
    input: &super::request::RequestInput,
    name: &str,
    args: &serde_json::Value,
) {
    if let Some(dir) = input.log_dir.as_deref() {
        let payload = format!(
            "tool_call: {name}\nargs: {}",
            serde_json::to_string_pretty(args).unwrap_or_default()
        );
        let _ = write_response_log(
            dir,
            &input.log_session_id,
            input.tab,
            input.message_index,
            &payload,
        );
    }
}

pub(super) fn handle_request_error(
    error: &str,
    input: &super::request::RequestInput,
    tx: &Sender<UiEvent>,
) {
    if let Some(dir) = input.log_dir.as_deref() {
        let payload = format!("error: {error}");
        let _ = write_response_log(
            dir,
            &input.log_session_id,
            input.tab,
            input.message_index,
            &payload,
        );
    }
    let _ = tx.send(UiEvent {
        tab: input.tab,
        request_id: input.request_id,
        event: LlmEvent::Error(error.to_string()),
    });
}
