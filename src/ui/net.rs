use crate::llm::rig::{build_completion_request, openai_completion_model, prepare_rig_context};
use crate::types::{Message, ToolCall, ToolFunctionCall, Usage};
use futures::StreamExt;
use rig::completion::{AssistantContent, GetTokenUsage};
use rig::streaming::StreamedAssistantContent;
use std::sync::mpsc::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::runtime::Runtime;

mod net_logging;
use net_logging::{build_enabled_tools, stream_chunks, write_request_log, write_response_log};

type OpenAiStreamResponse = rig::providers::openai::completion::streaming::StreamingCompletionResponse;
type RigStream = rig::streaming::StreamingCompletionResponse<OpenAiStreamResponse>;

pub enum LlmEvent {
    Chunk(String),
    Error(String),
    Done {
        usage: Option<Usage>,
    },
    ToolCalls {
        calls: Vec<ToolCall>,
        usage: Option<Usage>,
    },
}

pub struct UiEvent {
    pub tab: usize,
    pub request_id: u64,
    pub event: LlmEvent,
}

pub fn request_llm_stream(
    base_url: &str, api_key: &str, model: &str, messages: &[Message], prompts_dir: &str,
    enable_web_search: bool, enable_code_exec: bool, enable_read_file: bool,
    enable_read_code: bool, enable_modify_file: bool, log_dir: Option<String>,
    log_session_id: String, message_index: usize, cancel: Arc<AtomicBool>,
    tx: Sender<UiEvent>, tab: usize, request_id: u64,
) {
    let input = RequestInput::new(
        base_url,
        api_key,
        model,
        messages,
        prompts_dir,
        log_dir,
        log_session_id,
        message_index,
        tab,
        request_id,
    );
    let enabled = build_enabled_tools(
        enable_web_search,
        enable_code_exec,
        enable_read_file,
        enable_read_code,
        enable_modify_file,
    );
    run_llm_stream_with_input(input, enabled, cancel, tx);
}

fn run_llm_stream_with_input(
    input: RequestInput,
    enabled: Vec<&'static str>,
    cancel: Arc<AtomicBool>,
    tx: Sender<UiEvent>,
) {
    let Some(rt) = init_runtime(&tx, input.tab, input.request_id) else {
        return;
    };
    let result = rt.block_on(stream_request(&input, &enabled, &cancel, &tx));
    if let Err(err) = result {
        handle_request_error(&err, &input, &tx);
    }
}

struct RequestInput {
    base_url: String,
    api_key: String,
    model: String,
    messages: Vec<Message>,
    prompts_dir: String,
    log_dir: Option<String>,
    log_session_id: String,
    message_index: usize,
    tab: usize,
    request_id: u64,
}

impl RequestInput {
    fn new(
        base_url: &str,
        api_key: &str,
        model: &str,
        messages: &[Message],
        prompts_dir: &str,
        log_dir: Option<String>,
        log_session_id: String,
        message_index: usize,
        tab: usize,
        request_id: u64,
    ) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            messages: messages.to_vec(),
            prompts_dir: prompts_dir.to_string(),
            log_dir,
            log_session_id,
            message_index,
            tab,
            request_id,
        }
    }
}

fn init_runtime(tx: &Sender<UiEvent>, tab: usize, request_id: u64) -> Option<Runtime> {
    let rt = Runtime::new();
    if rt.is_err() {
        let _ = tx.send(UiEvent {
            tab,
            request_id,
            event: LlmEvent::Error("初始化 Tokio 失败".to_string()),
        });
        return None;
    }
    rt.ok()
}

async fn stream_request(
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
    usage: Option<Usage>,
) {
    log_response_text(input, text);
    stream_chunks(text, cancel, tx, input.tab, input.request_id);
    send_done(input, tx, usage);
}

fn handle_non_stream_tools(
    input: &RequestInput,
    tx: &Sender<UiEvent>,
    calls: &[rig::message::ToolCall],
    usage: Option<Usage>,
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

fn log_request(input: &RequestInput, ctx: &crate::llm::rig::RigRequestContext) {
    if let Some(dir) = input.log_dir.as_deref() {
        let _ = write_request_log(
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

fn log_response_text(input: &RequestInput, text: &str) {
    if let Some(dir) = input.log_dir.as_deref() {
        let _ = write_response_log(dir, &input.log_session_id, input.tab, input.message_index, text);
    }
}

fn send_chunk(tx: &Sender<UiEvent>, input: &RequestInput, chunk: String) {
    let _ = tx.send(UiEvent {
        tab: input.tab,
        request_id: input.request_id,
        event: LlmEvent::Chunk(chunk),
    });
}

fn send_done(input: &RequestInput, tx: &Sender<UiEvent>, usage: Option<Usage>) {
    let _ = tx.send(UiEvent {
        tab: input.tab,
        request_id: input.request_id,
        event: LlmEvent::Done { usage },
    });
}

fn send_tool_calls(
    input: &RequestInput,
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

fn convert_tool_call(
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

fn map_usage(usage: rig::completion::Usage) -> Usage {
    Usage {
        prompt_tokens: Some(usage.input_tokens),
        completion_tokens: Some(usage.output_tokens),
        total_tokens: Some(usage.total_tokens),
    }
}

fn usage_from_stream(res: &OpenAiStreamResponse) -> Option<Usage> {
    res.token_usage().map(map_usage)
}

fn log_tool_call(input: &RequestInput, name: &str, args: &serde_json::Value) {
    if let Some(dir) = input.log_dir.as_deref() {
        let payload = format!(
            "tool_call: {name}\nargs: {}",
            serde_json::to_string_pretty(args).unwrap_or_default()
        );
        let _ = write_response_log(dir, &input.log_session_id, input.tab, input.message_index, &payload);
    }
}

fn handle_request_error(error: &str, input: &RequestInput, tx: &Sender<UiEvent>) {
    if let Some(dir) = input.log_dir.as_deref() {
        let payload = format!("error: {error}");
        let _ = write_response_log(dir, &input.log_session_id, input.tab, input.message_index, &payload);
    }
    let _ = tx.send(UiEvent {
        tab: input.tab,
        request_id: input.request_id,
        event: LlmEvent::Error(error.to_string()),
    });
}

struct StreamState {
    text: String,
    usage: Option<Usage>,
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
