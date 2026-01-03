use crate::types::{Message, ROLE_ASSISTANT, ROLE_USER};
use crate::ui::net::{UiEvent, request_llm_stream};
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::{App, RequestHandle};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

pub(crate) fn start_tab_request(
    tab_state: &mut TabState,
    question: &str,
    base_url: &str,
    api_key: &str,
    model: &str,
    _show_reasoning: bool,
    tx: &mpsc::Sender<UiEvent>,
    tab_id: usize,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    log_requests: Option<String>,
    log_session_id: String,
) {
    let app = &mut tab_state.app;
    cancel_active_request(app);
    if !push_user_message(app, question) { return; }
    start_request_common(
        app, base_url, api_key, model, tx, tab_id, enable_web_search, enable_code_exec,
        enable_read_file, enable_read_code, enable_modify_file, log_requests, log_session_id,
    );
}

pub(crate) fn start_followup_request(
    tab_state: &mut TabState,
    base_url: &str,
    api_key: &str,
    model: &str,
    _show_reasoning: bool,
    tx: &mpsc::Sender<UiEvent>,
    tab_id: usize,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    log_requests: Option<String>,
    log_session_id: String,
) {
    let app = &mut tab_state.app;
    cancel_active_request(app);
    start_request_common(
        app, base_url, api_key, model, tx, tab_id, enable_web_search, enable_code_exec,
        enable_read_file, enable_read_code, enable_modify_file, log_requests, log_session_id,
    );
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
    Message { role: ROLE_USER.to_string(), content, tool_call_id: None, tool_calls: None }
}

fn start_request_common(
    app: &mut App,
    base_url: &str,
    api_key: &str,
    model: &str,
    tx: &mpsc::Sender<UiEvent>,
    tab_id: usize,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    log_requests: Option<String>,
    log_session_id: String,
) {
    if !ensure_api_key(app, api_key) { return; }
    let (messages, idx, request_id, cancel) = setup_request_state(app);
    let base_url = base_url.trim_end_matches('/').to_string();
    let api_key = api_key.to_string();
    let model = model.to_string();
    let prompts_dir = app.prompts_dir.clone();
    let tx = tx.clone();
    #[cfg(test)]
    if std::env::var("DEEPCHAT_TEST_SKIP_REQUEST").is_ok() { return; }
    spawn_llm_request(
        base_url, api_key, model, messages, prompts_dir, enable_web_search, enable_code_exec,
        enable_read_file, enable_read_code, enable_modify_file, log_requests, log_session_id, idx,
        cancel, tx, tab_id, request_id,
    );
}

fn spawn_llm_request(
    base_url: String, api_key: String, model: String, messages: Vec<Message>, prompts_dir: String,
    enable_web_search: bool, enable_code_exec: bool, enable_read_file: bool, enable_read_code: bool,
    enable_modify_file: bool, log_requests: Option<String>, log_session_id: String, idx: usize,
    cancel: Arc<AtomicBool>, tx: mpsc::Sender<UiEvent>, tab_id: usize, request_id: u64,
) {
    thread::spawn(move || request_llm_stream(
        &base_url, &api_key, &model, &messages, &prompts_dir, enable_web_search, enable_code_exec,
        enable_read_file, enable_read_code, enable_modify_file, log_requests, log_session_id, idx,
        cancel, tx, tab_id, request_id,
    ));
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
    app.active_request = Some(RequestHandle { id: request_id, cancel: Arc::clone(&cancel) });
    app.busy = true;
    app.busy_since = Some(Instant::now());
    app.pending_assistant = Some(idx);
    app.pending_reasoning = None;
    app.stream_buffer.clear();
    app.follow = true;
    app.dirty_indices.push(idx);
    (outbound_messages, idx, request_id, cancel)
}
