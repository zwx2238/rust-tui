use crate::types::{Message, ROLE_ASSISTANT};
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::{App, RequestHandle};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

pub(crate) struct StartTabRequestParams<'a> {
    pub tab_state: &'a mut TabState,
    pub question: &'a str,
    pub base_url: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
    pub max_tokens: Option<u64>,
    pub show_reasoning: bool,
    pub tx: &'a mpsc::Sender<RuntimeEvent>,
    pub enable_web_search: bool,
    pub enable_code_exec: bool,
    pub enable_read_file: bool,
    pub enable_read_code: bool,
    pub enable_modify_file: bool,
    pub enable_ask_questions: bool,
    pub log_requests: Option<String>,
    pub log_session_id: String,
}

pub(crate) fn start_tab_request(params: StartTabRequestParams<'_>) {
    let app = &mut params.tab_state.app;
    let tab_id = params.tab_state.conversation_id.clone();
    cancel_active_request(app);
    if !push_user_message(app, params.question) {
        return;
    }
    let default_role = app.default_role.clone();
    start_request_common(StartRequestCommonParams {
        app,
        base_url: params.base_url,
        api_key: params.api_key,
        model: params.model,
        max_tokens: params.max_tokens,
        show_reasoning: params.show_reasoning,
        tx: params.tx,
        tab_id,
        enable_web_search: params.enable_web_search,
        enable_code_exec: params.enable_code_exec,
        enable_read_file: params.enable_read_file,
        enable_read_code: params.enable_read_code,
        enable_modify_file: params.enable_modify_file,
        enable_ask_questions: params.enable_ask_questions,
        log_requests: params.log_requests,
        log_session_id: params.log_session_id,
        default_role,
    });
}

pub(crate) struct StartFollowupRequestParams<'a> {
    pub tab_state: &'a mut TabState,
    pub base_url: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
    pub max_tokens: Option<u64>,
    pub show_reasoning: bool,
    pub tx: &'a mpsc::Sender<RuntimeEvent>,
    pub enable_web_search: bool,
    pub enable_code_exec: bool,
    pub enable_read_file: bool,
    pub enable_read_code: bool,
    pub enable_modify_file: bool,
    pub enable_ask_questions: bool,
    pub log_requests: Option<String>,
    pub log_session_id: String,
}

pub(crate) fn start_followup_request(params: StartFollowupRequestParams<'_>) {
    let app = &mut params.tab_state.app;
    let tab_id = params.tab_state.conversation_id.clone();
    cancel_active_request(app);
    let default_role = app.default_role.clone();
    start_request_common(StartRequestCommonParams {
        app,
        base_url: params.base_url,
        api_key: params.api_key,
        model: params.model,
        max_tokens: params.max_tokens,
        show_reasoning: params.show_reasoning,
        tx: params.tx,
        tab_id,
        enable_web_search: params.enable_web_search,
        enable_code_exec: params.enable_code_exec,
        enable_read_file: params.enable_read_file,
        enable_read_code: params.enable_read_code,
        enable_modify_file: params.enable_modify_file,
        enable_ask_questions: params.enable_ask_questions,
        log_requests: params.log_requests,
        log_session_id: params.log_session_id,
        default_role,
    });
}

fn cancel_active_request(app: &mut App) {
    if let Some(handle) = &app.active_request {
        handle.cancel();
        app.active_request = None;
    }
}

fn push_user_message(app: &mut App, question: &str) -> bool {
    if !question.is_empty() {
        app.messages
            .push(user_message(&app.default_role, question.to_string()));
        return true;
    }
    if let Some(line) = app.pending_send.take() {
        app.messages.push(user_message(&app.default_role, line));
        return true;
    }
    false
}

fn user_message(role: &str, content: String) -> Message {
    Message {
        role: role.to_string(),
        content,
        tool_call_id: None,
        tool_calls: None,
    }
}

struct StartRequestCommonParams<'a> {
    app: &'a mut App,
    base_url: &'a str,
    api_key: &'a str,
    model: &'a str,
    max_tokens: Option<u64>,
    show_reasoning: bool,
    tx: &'a mpsc::Sender<RuntimeEvent>,
    tab_id: String,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    enable_ask_questions: bool,
    log_requests: Option<String>,
    log_session_id: String,
    default_role: String,
}

fn start_request_common(params: StartRequestCommonParams<'_>) {
    if !ensure_api_key(params.app, params.api_key) {
        return;
    }
    let state = build_request_state(params.app);
    spawn_llm_request(build_spawn_params(params, state));
}

struct RequestState {
    messages: Vec<Message>,
    idx: usize,
    cancel: Arc<AtomicBool>,
    request_id: u64,
    prompts_dir: String,
}

fn build_request_state(app: &mut App) -> RequestState {
    let (messages, idx, request_id, cancel) = setup_request_state(app);
    RequestState {
        messages,
        idx,
        cancel,
        request_id,
        prompts_dir: app.prompts_dir.clone(),
    }
}

fn build_spawn_params(
    params: StartRequestCommonParams<'_>,
    state: RequestState,
) -> SpawnLlmRequestParams {
    SpawnLlmRequestParams {
        base_url: params.base_url.trim_end_matches('/').to_string(),
        api_key: params.api_key.to_string(),
        model: params.model.to_string(),
        max_tokens: params.max_tokens,
        messages: state.messages,
        prompts_dir: state.prompts_dir,
        show_reasoning: params.show_reasoning,
        enable_web_search: params.enable_web_search,
        enable_code_exec: params.enable_code_exec,
        enable_read_file: params.enable_read_file,
        enable_read_code: params.enable_read_code,
        enable_modify_file: params.enable_modify_file,
        enable_ask_questions: params.enable_ask_questions,
        log_requests: params.log_requests,
        log_session_id: params.log_session_id,
        default_role: params.default_role,
        idx: state.idx,
        cancel: state.cancel,
        tx: params.tx.clone(),
        tab_id: params.tab_id,
        request_id: state.request_id,
    }
}

struct SpawnLlmRequestParams {
    base_url: String,
    api_key: String,
    model: String,
    max_tokens: Option<u64>,
    messages: Vec<Message>,
    prompts_dir: String,
    show_reasoning: bool,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    enable_ask_questions: bool,
    log_requests: Option<String>,
    log_session_id: String,
    default_role: String,
    idx: usize,
    cancel: Arc<AtomicBool>,
    tx: mpsc::Sender<RuntimeEvent>,
    tab_id: String,
    request_id: u64,
}

fn spawn_llm_request(params: SpawnLlmRequestParams) {
    thread::spawn(move || {
        crate::services::net::request_llm_stream(crate::services::net::LlmStreamRequestParams {
            base_url: params.base_url,
            api_key: params.api_key,
            model: params.model,
            max_tokens: params.max_tokens,
            messages: params.messages,
            prompts_dir: params.prompts_dir,
            show_reasoning: params.show_reasoning,
            enable_web_search: params.enable_web_search,
            enable_code_exec: params.enable_code_exec,
            enable_read_file: params.enable_read_file,
            enable_read_code: params.enable_read_code,
            enable_modify_file: params.enable_modify_file,
            enable_ask_questions: params.enable_ask_questions,
            log_dir: params.log_requests,
            log_session_id: params.log_session_id,
            default_role: params.default_role,
            message_index: params.idx,
            cancel: params.cancel,
            tx: params.tx,
            tab: params.tab_id,
            request_id: params.request_id,
        })
    });
}

fn ensure_api_key(app: &mut App, api_key: &str) -> bool {
    if !api_key.trim().is_empty() {
        return true;
    }
    app.messages.push(Message {
        role: ROLE_ASSISTANT.to_string(),
        content: "缺少 API Key，无法请求模型。".to_string(),
        tool_call_id: None,
        tool_calls: None,
    });
    false
}

fn setup_request_state(app: &mut App) -> (Vec<Message>, usize, u64, Arc<AtomicBool>) {
    let outbound_messages = app.messages.clone();
    let idx = app.messages.len();
    app.messages.push(Message {
        role: ROLE_ASSISTANT.to_string(),
        content: String::new(),
        tool_call_id: None,
        tool_calls: None,
    });
    let request_id = app.next_request_id;
    app.next_request_id = app.next_request_id.saturating_add(1);
    let cancel = Arc::new(AtomicBool::new(false));
    app.active_request = Some(RequestHandle {
        id: request_id,
        cancel: Arc::clone(&cancel),
    });
    app.busy = true;
    app.busy_since = Some(Instant::now());
    app.pending_assistant = Some(idx);
    app.pending_reasoning = None;
    app.stream_buffer.clear();
    app.dirty_indices.push(idx);
    (outbound_messages, idx, request_id, cancel)
}
