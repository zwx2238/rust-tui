use crate::llm::rig::{CompletionModelChoice, build_completion_request, completion_model_for, prepare_rig_context};
use crate::types::ToolCall;
use crate::ui::events::RuntimeEvent;
use futures::StreamExt;
use rig::completion::CompletionModel;
use rig::completion::AssistantContent;
use rig::streaming::StreamedAssistantContent;
use std::sync::mpsc::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::helpers::{
    convert_tool_call, log_request, log_response_text, log_tool_call, map_usage, send_chunk,
    send_done, send_reasoning_chunk, send_tool_calls, usage_from_stream,
};
use super::request::RequestInput;

pub(super) async fn stream_request(
    input: &RequestInput,
    enabled: &[&'static str],
    cancel: &Arc<AtomicBool>,
    tx: &Sender<RuntimeEvent>,
) -> Result<(), String> {
    let (ctx, _templates) = prepare_rig_context(&input.messages, &input.prompts_dir, enabled)?;
    log_request(input, &ctx);
    let model = completion_model_for(&input.base_url, &input.api_key, &input.model)?;
    stream_with_model(model, &ctx, input, cancel, tx).await
}

async fn stream_with_model(
    model: CompletionModelChoice,
    ctx: &crate::llm::rig::RigRequestContext,
    input: &RequestInput,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<RuntimeEvent>,
) -> Result<(), String> {
    match model {
        CompletionModelChoice::OpenAi(model) => {
            stream_with_model_impl(model, ctx, input, cancel, tx, input.max_tokens).await
        }
        CompletionModelChoice::DeepSeek(model) => {
            stream_with_model_impl(model, ctx, input, cancel, tx, input.max_tokens).await
        }
        CompletionModelChoice::Anthropic(model) => {
            let tokens = input.max_tokens.or(Some(1024));
            stream_with_model_impl(model, ctx, input, cancel, tx, tokens).await
        }
    }
}

async fn stream_with_model_impl<M>(
    model: M,
    ctx: &crate::llm::rig::RigRequestContext,
    input: &RequestInput,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<RuntimeEvent>,
    max_tokens: Option<u64>,
) -> Result<(), String>
where
    M: CompletionModel,
{
    let stream = match build_completion_request(&model, ctx)
        .max_tokens_opt(max_tokens)
        .stream()
        .await
    {
        Ok(stream) => stream,
        Err(_) => {
            return run_non_stream_request(&model, ctx, input, cancel, tx, max_tokens).await;
        }
    };
    process_stream(stream, input, cancel, tx).await
}

async fn process_stream<R>(
    mut stream: rig::streaming::StreamingCompletionResponse<R>,
    input: &RequestInput,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<RuntimeEvent>,
) -> Result<(), String>
where
    R: rig::completion::GetTokenUsage + Clone + Unpin,
{
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

fn handle_stream_item<R>(
    item: Result<StreamedAssistantContent<R>, rig::completion::CompletionError>,
    state: &mut StreamState,
    input: &RequestInput,
    tx: &Sender<RuntimeEvent>,
) -> Result<StreamStep, String>
where
    R: rig::completion::GetTokenUsage,
{
    match item {
        Ok(content) => handle_stream_ok(content, state, input, tx),
        Err(err) => Err(format!("请求失败：{err}")),
    }
}

fn handle_stream_ok<R>(
    content: StreamedAssistantContent<R>,
    state: &mut StreamState,
    input: &RequestInput,
    tx: &Sender<RuntimeEvent>,
) -> Result<StreamStep, String>
where
    R: rig::completion::GetTokenUsage,
{
    match content {
        StreamedAssistantContent::Text(text) => {
            state.text.push_str(&text.text);
            send_chunk(tx, input, text.text);
            Ok(StreamStep::Continue)
        }
        StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
            state.seen_reasoning_delta = true;
            send_reasoning_if_enabled(input, tx, reasoning);
            Ok(StreamStep::Continue)
        }
        StreamedAssistantContent::Reasoning(reasoning) => {
            let text = reasoning.reasoning.join("");
            send_final_reasoning_if_needed(state, input, tx, text);
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
    model: &impl CompletionModel,
    ctx: &crate::llm::rig::RigRequestContext,
    input: &RequestInput,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<RuntimeEvent>,
    max_tokens: Option<u64>,
) -> Result<(), String> {
    let response = build_completion_request(model, ctx)
        .max_tokens_opt(max_tokens)
        .send()
        .await
        .map_err(|e| format!("请求失败：{e}"))?;
    if cancel.load(Ordering::Relaxed) {
        return Ok(());
    }
    let (text, calls, reasoning) = split_choice(&response.choice);
    let usage = Some(map_usage(response.usage));
    send_reasoning_from_choice(input, tx, &reasoning);
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
    tx: &Sender<RuntimeEvent>,
    text: &str,
    usage: Option<crate::types::Usage>,
) {
    log_response_text(input, text);
    super::net_logging::stream_chunks(text, cancel, tx, input.tab, input.request_id);
    send_done(input, tx, usage);
}

fn handle_non_stream_tools(
    input: &RequestInput,
    tx: &Sender<RuntimeEvent>,
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
) -> (String, Vec<rig::message::ToolCall>, Option<String>) {
    let mut text = String::new();
    let mut calls = Vec::new();
    let mut reasoning = None;
    for item in choice.iter() {
        match item {
            AssistantContent::Text(t) => text.push_str(&t.text),
            AssistantContent::ToolCall(call) => calls.push(call.clone()),
            AssistantContent::Reasoning(r) => {
                reasoning = Some(r.reasoning.join(""));
            }
            _ => {}
        }
    }
    (text, calls, reasoning)
}

fn finalize_stream(input: &RequestInput, tx: &Sender<RuntimeEvent>, state: StreamState) {
    log_response_text(input, &state.text);
    send_done(input, tx, state.usage);
}

struct StreamState {
    text: String,
    usage: Option<crate::types::Usage>,
    seen_reasoning_delta: bool,
}

impl StreamState {
    fn new() -> Self {
        Self {
            text: String::new(),
            usage: None,
            seen_reasoning_delta: false,
        }
    }
}

enum StreamStep {
    Continue,
    ToolCalls(Vec<ToolCall>),
}

fn send_reasoning_if_enabled(input: &RequestInput, tx: &Sender<RuntimeEvent>, text: String) {
    if input.show_reasoning && !text.is_empty() {
        send_reasoning_chunk(input, tx, text);
    }
}

fn send_final_reasoning_if_needed(
    state: &StreamState,
    input: &RequestInput,
    tx: &Sender<RuntimeEvent>,
    text: String,
) {
    if state.seen_reasoning_delta {
        return;
    }
    send_reasoning_if_enabled(input, tx, text);
}

fn send_reasoning_from_choice(
    input: &RequestInput,
    tx: &Sender<RuntimeEvent>,
    reasoning: &Option<String>,
) {
    if let Some(text) = reasoning {
        send_reasoning_if_enabled(input, tx, text.clone());
    }
}
