use crate::llm::rig::{build_completion_request, openai_completion_model, prepare_rig_context};
use futures::StreamExt;
use crate::types::ToolCall;
use rig::completion::AssistantContent;
use rig::streaming::StreamedAssistantContent;
use std::sync::mpsc::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::helpers::{
    convert_tool_call, log_request, log_response_text, log_tool_call, map_usage, send_chunk,
    send_done, send_tool_calls, usage_from_stream,
};
use super::request::RequestInput;
use super::types::UiEvent;

pub(super) type OpenAiStreamResponse =
    rig::providers::openai::completion::streaming::StreamingCompletionResponse;
type RigStream = rig::streaming::StreamingCompletionResponse<OpenAiStreamResponse>;

pub(super) async fn stream_request(
    input: &RequestInput,
    enabled: &[&'static str],
    cancel: &Arc<AtomicBool>,
    tx: &Sender<UiEvent>,
) -> Result<(), String> {
    let (ctx, _templates) = prepare_rig_context(&input.messages, &input.prompts_dir, enabled)?;
    log_request(input, &ctx);
    let model = openai_completion_model(&input.base_url, &input.api_key, &input.model)?;
    let stream = match build_completion_request(&model, &ctx).stream().await {
        Ok(stream) => stream,
        Err(_) => {
            return run_non_stream_request(&model, &ctx, input, cancel, tx).await;
        }
    };
    process_stream(stream, input, cancel, tx).await
}

async fn process_stream(
    mut stream: RigStream,
    input: &RequestInput,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<UiEvent>,
) -> Result<(), String> {
    let mut state = StreamState::new();
    while let Some(item) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            stream.cancel();
            return Ok(());
        }
        match handle_stream_item(item, &mut state, input, tx)? {
            StreamStep::Continue => {}
            StreamStep::ToolCalls(calls) => {
                send_tool_calls(input, tx, calls, state.usage.take());
                return Ok(());
            }
        }
    }
    finalize_stream(input, tx, state);
    Ok(())
}

fn handle_stream_item(
    item: Result<StreamedAssistantContent<OpenAiStreamResponse>, rig::completion::CompletionError>,
    state: &mut StreamState,
    input: &RequestInput,
    tx: &Sender<UiEvent>,
) -> Result<StreamStep, String> {
    match item {
        Ok(content) => handle_stream_ok(content, state, input, tx),
        Err(err) => Err(format!("请求失败：{err}")),
    }
}

fn handle_stream_ok(
    content: StreamedAssistantContent<OpenAiStreamResponse>,
    state: &mut StreamState,
    input: &RequestInput,
    tx: &Sender<UiEvent>,
) -> Result<StreamStep, String> {
    match content {
        StreamedAssistantContent::Text(text) => {
            state.text.push_str(&text.text);
            send_chunk(tx, input, text.text);
            Ok(StreamStep::Continue)
        }
        StreamedAssistantContent::ToolCall(call) => {
            log_tool_call(input, &call.function.name, &call.function.arguments);
            send_chunk(tx, input, format!("调用工具：{}\n", call.function.name));
            let calls = vec![convert_tool_call(&call, input.tab, input.request_id)];
            Ok(StreamStep::ToolCalls(calls))
        }
        StreamedAssistantContent::Final(res) => {
            state.usage = usage_from_stream(&res);
            Ok(StreamStep::Continue)
        }
        _ => Ok(StreamStep::Continue),
    }
}

async fn run_non_stream_request(
    model: &rig::providers::openai::completion::CompletionModel,
    ctx: &crate::llm::rig::RigRequestContext,
    input: &RequestInput,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<UiEvent>,
) -> Result<(), String> {
    let response = build_completion_request(model, ctx)
        .send()
        .await
        .map_err(|e| format!("请求失败：{e}"))?;
    if cancel.load(Ordering::Relaxed) {
        return Ok(());
    }
    let (text, calls) = split_choice(&response.choice);
    let usage = Some(map_usage(response.usage));
    if calls.is_empty() {
        handle_non_stream_text(input, cancel, tx, &text, usage);
        return Ok(());
    }
    handle_non_stream_tools(input, tx, &calls, usage);
    Ok(())
}

fn handle_non_stream_text(
    input: &RequestInput,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<UiEvent>,
    text: &str,
    usage: Option<crate::types::Usage>,
) {
    log_response_text(input, text);
    super::net_logging::stream_chunks(text, cancel, tx, input.tab, input.request_id);
    send_done(input, tx, usage);
}

fn handle_non_stream_tools(
    input: &RequestInput,
    tx: &Sender<UiEvent>,
    calls: &[rig::message::ToolCall],
    usage: Option<crate::types::Usage>,
) {
    for call in calls {
        log_tool_call(input, &call.function.name, &call.function.arguments);
    }
    let mapped = calls
        .iter()
        .map(|call| convert_tool_call(call, input.tab, input.request_id))
        .collect();
    send_tool_calls(input, tx, mapped, usage);
}

fn split_choice(
    choice: &rig::OneOrMany<AssistantContent>,
) -> (String, Vec<rig::message::ToolCall>) {
    let mut text = String::new();
    let mut calls = Vec::new();
    for item in choice.iter() {
        match item {
            AssistantContent::Text(t) => text.push_str(&t.text),
            AssistantContent::ToolCall(call) => calls.push(call.clone()),
            _ => {}
        }
    }
    (text, calls)
}

fn finalize_stream(input: &RequestInput, tx: &Sender<UiEvent>, state: StreamState) {
    log_response_text(input, &state.text);
    send_done(input, tx, state.usage);
}

struct StreamState {
    text: String,
    usage: Option<crate::types::Usage>,
}

impl StreamState {
    fn new() -> Self {
        Self {
            text: String::new(),
            usage: None,
        }
    }
}

enum StreamStep {
    Continue,
    ToolCalls(Vec<ToolCall>),
}
