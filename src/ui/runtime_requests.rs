use crate::types::{Message, ROLE_ASSISTANT, ROLE_USER};
use crate::ui::net::{UiEvent, request_llm_stream};
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::RequestHandle;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
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
    log_requests: Option<String>,
) {
    let app = &mut tab_state.app;
    if let Some(handle) = &app.active_request {
        handle.cancel();
        app.active_request = None;
    }
    if !question.is_empty() {
        app.messages.push(Message {
            role: ROLE_USER.to_string(),
            content: question.to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
    } else if let Some(line) = app.pending_send.take() {
        app.messages.push(Message {
            role: ROLE_USER.to_string(),
            content: line,
            tool_call_id: None,
            tool_calls: None,
        });
    } else {
        return;
    }
    if api_key.trim().is_empty() {
        app.messages.push(Message {
            role: ROLE_ASSISTANT.to_string(),
            content: "缺少 API Key，无法请求模型。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        return;
    }
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
    let messages = outbound_messages;
    let base_url = base_url.trim_end_matches('/').to_string();
    let api_key = api_key.to_string();
    let model = model.to_string();
    let prompts_dir = app.prompts_dir.clone();
    let tx = tx.clone();
    thread::spawn(move || {
        request_llm_stream(
            &base_url,
            &api_key,
            &model,
            &messages,
            &prompts_dir,
            enable_web_search,
            enable_code_exec,
            log_requests,
            cancel,
            tx,
            tab_id,
            request_id,
        );
    });
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
    log_requests: Option<String>,
) {
    let app = &mut tab_state.app;
    if let Some(handle) = &app.active_request {
        handle.cancel();
        app.active_request = None;
    }
    if api_key.trim().is_empty() {
        app.messages.push(Message {
            role: ROLE_ASSISTANT.to_string(),
            content: "缺少 API Key，无法请求模型。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        return;
    }
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
    let messages = outbound_messages;
    let base_url = base_url.trim_end_matches('/').to_string();
    let api_key = api_key.to_string();
    let model = model.to_string();
    let prompts_dir = app.prompts_dir.clone();
    let tx = tx.clone();
    thread::spawn(move || {
        request_llm_stream(
            &base_url,
            &api_key,
            &model,
            &messages,
            &prompts_dir,
            enable_web_search,
            enable_code_exec,
            log_requests,
            cancel,
            tx,
            tab_id,
            request_id,
        );
    });
}
