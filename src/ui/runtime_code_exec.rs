use crate::args::Args;
use crate::types::Message;
use crate::ui::code_exec_container::{ensure_container, run_python_in_container_stream};
use crate::ui::net::UiEvent;
use crate::ui::runtime_code_exec_helpers::inject_requirements;
use crate::ui::runtime_code_exec_output::{escape_json_string, take_code_exec_reason};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_followup_request;
use crate::ui::state::{App, CodeExecLive, CodeExecReasonTarget, PendingCodeExec};
use crate::ui::tools::parse_code_exec_args;
use std::sync::mpsc;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn handle_code_exec_request(
    tab_state: &mut TabState,
    call: &crate::types::ToolCall,
) -> Result<(), String> {
    if tab_state.app.pending_code_exec.is_some() {
        return Err("已有待审批的代码执行请求".to_string());
    }
    let request = parse_code_exec_args(&call.function.arguments)?;
    tab_state.app.pending_code_exec = Some(PendingCodeExec {
        call_id: call.id.clone(),
        language: request.language,
        code: request.code,
        exec_code: None,
        requested_at: Instant::now(),
        stop_reason: None,
    });
    reset_code_exec_ui(&mut tab_state.app);
    Ok(())
}

pub(crate) fn handle_code_exec_stop(tab_state: &mut TabState) {
    let reason = take_code_exec_reason(tab_state, CodeExecReasonTarget::Stop)
        .unwrap_or_else(|| "用户中止".to_string());
    if let Some(pending) = tab_state.app.pending_code_exec.as_mut() {
        pending.stop_reason = Some(reason);
    }
    if let Some(cancel) = &tab_state.app.code_exec_cancel {
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    tab_state.app.code_exec_hover = None;
}

pub(crate) fn handle_code_exec_exit(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some((pending, content)) = take_pending_and_output(&mut tab_state.app) else { return; };
    push_tool_message(&mut tab_state.app, content, pending.call_id);
    reset_code_exec_after_exit(&mut tab_state.app);
    start_followup(tab_state, tab_id, registry, args, tx);
}

pub(crate) fn handle_code_exec_approve(
    tab_state: &mut TabState,
    _tab_id: usize,
    _registry: &crate::model_registry::ModelRegistry,
    _args: &Args,
    _tx: &mpsc::Sender<UiEvent>,
) {
    let Some(mut pending) = clone_pending_or_notify(&mut tab_state.app) else { return; };
    if tab_state.app.code_exec_live.is_some() { return; }
    let live = init_code_exec_live(&mut tab_state.app);
    let cancel = init_cancel_flag(&mut tab_state.app);
    let run_id = init_run_id(&mut tab_state.app);
    let exec_code = inject_requirements(&pending.code);
    pending.exec_code = Some(exec_code.clone());
    if let Some(current) = tab_state.app.pending_code_exec.as_mut() {
        current.exec_code = Some(exec_code);
    }
    let container_id = match ensure_container(&mut tab_state.app.code_exec_container_id) {
        Ok(id) => id,
        Err(err) => { mark_exec_error(&live, err); return; }
    };
    if pending.language == "python" {
        spawn_python_exec(container_id, run_id, pending, live, cancel);
    } else {
        mark_unsupported_language(&live, &pending.language);
    }
}

pub(crate) fn handle_code_exec_deny(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some(pending) = take_pending_or_notify(&mut tab_state.app) else { return; };
    reset_code_exec_after_deny(&mut tab_state.app);
    let reason = take_code_exec_reason(tab_state, CodeExecReasonTarget::Deny)
        .unwrap_or_else(|| "用户取消".to_string());
    let content = format!(r#"{{"error":"用户拒绝执行","reason":"{}"}}"#, escape_json_string(&reason));
    push_tool_message(&mut tab_state.app, content, pending.call_id);
    start_followup(tab_state, tab_id, registry, args, tx);
}

fn take_pending_or_notify(app: &mut App) -> Option<PendingCodeExec> {
    let Some(pending) = app.pending_code_exec.take() else { push_no_pending(app); return None; };
    Some(pending)
}

fn clone_pending_or_notify(app: &mut App) -> Option<PendingCodeExec> {
    let Some(pending) = app.pending_code_exec.clone() else { push_no_pending(app); return None; };
    Some(pending)
}

fn push_no_pending(app: &mut App) {
    let idx = app.messages.len();
    app.messages.push(Message {
        role: crate::types::ROLE_ASSISTANT.to_string(),
        content: "没有待审批的代码执行请求。".to_string(),
        tool_call_id: None,
        tool_calls: None,
    });
    app.dirty_indices.push(idx);
}

fn reset_code_exec_ui(app: &mut App) {
    app.code_exec_scroll = 0;
    app.code_exec_stdout_scroll = 0;
    app.code_exec_stderr_scroll = 0;
    app.code_exec_run_id = None;
    app.code_exec_result_ready = false;
    app.code_exec_finished_output = None;
    app.code_exec_cancel = None;
    app.code_exec_hover = None;
    app.code_exec_reason_target = None;
    app.code_exec_reason_input = tui_textarea::TextArea::default();
}

fn init_code_exec_live(app: &mut App) -> std::sync::Arc<std::sync::Mutex<CodeExecLive>> {
    let live = std::sync::Arc::new(std::sync::Mutex::new(CodeExecLive {
        started_at: std::time::Instant::now(),
        finished_at: None,
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
        done: false,
    }));
    app.code_exec_live = Some(live.clone());
    app.code_exec_result_ready = false;
    app.code_exec_finished_output = None;
    app.code_exec_hover = None;
    app.code_exec_reason_target = None;
    app.code_exec_reason_input = tui_textarea::TextArea::default();
    app.code_exec_scroll = 0;
    app.code_exec_stdout_scroll = 0;
    app.code_exec_stderr_scroll = 0;
    app.code_exec_run_id = None;
    live
}

fn init_cancel_flag(app: &mut App) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    app.code_exec_cancel = Some(cancel.clone());
    cancel
}

fn init_run_id(app: &mut App) -> String {
    let run_id = format!(
        "run-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
    );
    app.code_exec_run_id = Some(run_id.clone());
    run_id
}

fn spawn_python_exec(
    container_id: String,
    run_id: String,
    pending: PendingCodeExec,
    live: std::sync::Arc<std::sync::Mutex<CodeExecLive>>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    std::thread::spawn(move || {
        let code = pending.exec_code.as_deref().unwrap_or(&pending.code);
        if let Err(err) = run_python_in_container_stream(&container_id, &run_id, code, live.clone(), cancel) {
            mark_exec_error(&live, err);
        }
    });
}

fn mark_exec_error(live: &std::sync::Arc<std::sync::Mutex<CodeExecLive>>, err: String) {
    if let Ok(mut live) = live.lock() {
        live.stderr.push_str(&format!("{err}\n"));
        live.exit_code = Some(-1);
        live.done = true;
        live.finished_at = Some(std::time::Instant::now());
    }
}

fn mark_unsupported_language(live: &std::sync::Arc<std::sync::Mutex<CodeExecLive>>, language: &str) {
    if let Ok(mut live) = live.lock() {
        live.stderr = format!("不支持的语言：{}", language);
        live.exit_code = Some(-1);
        live.done = true;
        live.finished_at = Some(std::time::Instant::now());
    }
}

fn take_pending_and_output(app: &mut App) -> Option<(PendingCodeExec, String)> {
    let pending = app.pending_code_exec.take()?;
    let content = app.code_exec_finished_output.take()?;
    Some((pending, content))
}

fn push_tool_message(app: &mut App, content: String, call_id: String) {
    let idx = app.messages.len();
    app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content,
        tool_call_id: Some(call_id),
        tool_calls: None,
    });
    app.dirty_indices.push(idx);
}

fn reset_code_exec_after_exit(app: &mut App) {
    app.code_exec_live = None;
    app.code_exec_result_ready = false;
    app.code_exec_cancel = None;
    app.code_exec_hover = None;
    app.code_exec_reason_target = None;
    app.code_exec_reason_input = tui_textarea::TextArea::default();
    app.code_exec_scroll = 0;
    app.code_exec_stdout_scroll = 0;
    app.code_exec_stderr_scroll = 0;
    app.code_exec_run_id = None;
}

fn reset_code_exec_after_deny(app: &mut App) {
    app.code_exec_live = None;
    app.code_exec_result_ready = false;
    app.code_exec_finished_output = None;
    app.code_exec_cancel = None;
    app.code_exec_hover = None;
    app.code_exec_scroll = 0;
    app.code_exec_stdout_scroll = 0;
    app.code_exec_stderr_scroll = 0;
}

fn start_followup(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let model = registry
        .get(&tab_state.app.model_key)
        .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"));
    start_followup_request(
        tab_state,
        &model.base_url,
        &model.api_key,
        &model.model,
        args.show_reasoning,
        tx,
        tab_id,
        args.web_search_enabled(),
        args.code_exec_enabled(),
        args.read_file_enabled(),
        args.read_code_enabled(),
        args.modify_file_enabled(),
        args.log_requests.clone(),
        tab_state.app.log_session_id.clone(),
    );
}
