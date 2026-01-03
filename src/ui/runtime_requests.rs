use crate::types::{Message, ROLE_ASSISTANT, ROLE_USER};
use crate::ui::net::UiEvent;
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
    pub _show_reasoning: bool,
    pub tx: &'a mpsc::Sender<UiEvent>,
    pub tab_id: usize,
    pub enable_web_search: bool,
    pub enable_code_exec: bool,
    pub enable_read_file: bool,
    pub enable_read_code: bool,
    pub enable_modify_file: bool,
    pub log_requests: Option<String>,
    pub log_session_id: String,
}

pub(crate) fn start_tab_request(params: StartTabRequestParams<'_>) {
    let app = &mut params.tab_state.app;
    cancel_active_request(app);
    if !push_user_message(app, params.question) {
        return;
    }
    start_request_common(StartRequestCommonParams {
        app,
        base_url: params.base_url,
        api_key: params.api_key,
        model: params.model,
        tx: params.tx,
        tab_id: params.tab_id,
        enable_web_search: params.enable_web_search,
        enable_code_exec: params.enable_code_exec,
        enable_read_file: params.enable_read_file,
        enable_read_code: params.enable_read_code,
        enable_modify_file: params.enable_modify_file,
        log_requests: params.log_requests,
        log_session_id: params.log_session_id,
    });
}

pub(crate) struct StartFollowupRequestParams<'a> {
    pub tab_state: &'a mut TabState,
    pub base_url: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
    pub _show_reasoning: bool,
    pub tx: &'a mpsc::Sender<UiEvent>,
    pub tab_id: usize,
    pub enable_web_search: bool,
    pub enable_code_exec: bool,
    pub enable_read_file: bool,
    pub enable_read_code: bool,
    pub enable_modify_file: bool,
    pub log_requests: Option<String>,
    pub log_session_id: String,
}

pub(crate) fn start_followup_request(params: StartFollowupRequestParams<'_>) {
    let app = &mut params.tab_state.app;
    cancel_active_request(app);
    start_request_common(StartRequestCommonParams {
        app,
        base_url: params.base_url,
        api_key: params.api_key,
        model: params.model,
        tx: params.tx,
        tab_id: params.tab_id,
        enable_web_search: params.enable_web_search,
        enable_code_exec: params.enable_code_exec,
        enable_read_file: params.enable_read_file,
        enable_read_code: params.enable_read_code,
        enable_modify_file: params.enable_modify_file,
        log_requests: params.log_requests,
        log_session_id: params.log_session_id,
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
        app.messages.push(user_message(question.to_string()));
        return true;
    }
    if let Some(line) = app.pending_send.take() {
        app.messages.push(user_message(line));
        return true;
    }
    false
}

fn user_message(content: String) -> Message {
    Message {
        role: ROLE_USER.to_string(),
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
    tx: &'a mpsc::Sender<UiEvent>,
    tab_id: usize,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    log_requests: Option<String>,
    log_session_id: String,
}

fn start_request_common(params: StartRequestCommonParams<'_>) {
    if !ensure_api_key(params.app, params.api_key) {
        return;
    }
    let (messages, idx, request_id, cancel) = setup_request_state(params.app);
    let base_url = params.base_url.trim_end_matches('/').to_string();
    let api_key = params.api_key.to_string();
    let model = params.model.to_string();
    let prompts_dir = params.app.prompts_dir.clone();
    let tx = params.tx.clone();
    #[cfg(test)]
    if std::env::var("DEEPCHAT_TEST_SKIP_REQUEST").is_ok() {
        return;
    }
    spawn_llm_request(SpawnLlmRequestParams {
        base_url,
        api_key,
        model,
        messages,
        prompts_dir,
        enable_web_search: params.enable_web_search,
        enable_code_exec: params.enable_code_exec,
        enable_read_file: params.enable_read_file,
        enable_read_code: params.enable_read_code,
        enable_modify_file: params.enable_modify_file,
        log_requests: params.log_requests,
        log_session_id: params.log_session_id,
        idx,
        cancel,
        tx,
        tab_id: params.tab_id,
        request_id,
    });
}

struct SpawnLlmRequestParams {
    base_url: String,
    api_key: String,
    model: String,
    messages: Vec<Message>,
    prompts_dir: String,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    log_requests: Option<String>,
    log_session_id: String,
    idx: usize,
    cancel: Arc<AtomicBool>,
    tx: mpsc::Sender<UiEvent>,
    tab_id: usize,
    request_id: u64,
}

fn spawn_llm_request(params: SpawnLlmRequestParams) {
    thread::spawn(move || {
        crate::ui::net::request_llm_stream(crate::ui::net::LlmStreamRequestParams {
            base_url: params.base_url,
            api_key: params.api_key,
            model: params.model,
            messages: params.messages,
            prompts_dir: params.prompts_dir,
            enable_web_search: params.enable_web_search,
            enable_code_exec: params.enable_code_exec,
            enable_read_file: params.enable_read_file,
            enable_read_code: params.enable_read_code,
            enable_modify_file: params.enable_modify_file,
            log_dir: params.log_requests,
            log_session_id: params.log_session_id,
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
    app.follow = true;
    app.dirty_indices.push(idx);
    (outbound_messages, idx, request_id, cancel)
}
