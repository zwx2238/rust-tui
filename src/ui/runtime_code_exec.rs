use crate::args::Args;
use crate::ui::code_exec_container::{ensure_container, run_python_in_container_stream};
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_followup_request;
use crate::ui::runtime_code_exec_helpers::inject_requirements;
use crate::ui::runtime_code_exec_output::{escape_json_string, take_code_exec_reason};
use crate::ui::state::{CodeExecLive, CodeExecReasonTarget, PendingCodeExec};
use crate::ui::tools::parse_code_exec_args;
use crate::types::Message;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Instant;

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
    tab_state.app.code_exec_scroll = 0;
    tab_state.app.code_exec_stdout_scroll = 0;
    tab_state.app.code_exec_stderr_scroll = 0;
    tab_state.app.code_exec_run_id = None;
    tab_state.app.code_exec_result_ready = false;
    tab_state.app.code_exec_finished_output = None;
    tab_state.app.code_exec_cancel = None;
    tab_state.app.code_exec_hover = None;
    tab_state.app.code_exec_reason_target = None;
    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
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
    let Some(pending) = tab_state.app.pending_code_exec.take() else {
        return;
    };
    let Some(content) = tab_state.app.code_exec_finished_output.take() else {
        return;
    };
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content,
        tool_call_id: Some(pending.call_id),
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
    tab_state.app.code_exec_live = None;
    tab_state.app.code_exec_result_ready = false;
    tab_state.app.code_exec_cancel = None;
    tab_state.app.code_exec_hover = None;
    tab_state.app.code_exec_reason_target = None;
    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
    tab_state.app.code_exec_scroll = 0;
    tab_state.app.code_exec_stdout_scroll = 0;
    tab_state.app.code_exec_stderr_scroll = 0;
    tab_state.app.code_exec_run_id = None;
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
        args.log_requests.clone(),
        tab_state.app.log_session_id.clone(),
    );
}

pub(crate) fn handle_code_exec_approve(
    tab_state: &mut TabState,
    _tab_id: usize,
    _registry: &crate::model_registry::ModelRegistry,
    _args: &Args,
    _tx: &mpsc::Sender<UiEvent>,
) {
    let Some(mut pending) = tab_state.app.pending_code_exec.clone() else {
        let idx = tab_state.app.messages.len();
        tab_state.app.messages.push(Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "没有待审批的代码执行请求。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab_state.app.dirty_indices.push(idx);
        return;
    };
    if tab_state.app.code_exec_live.is_some() {
        return;
    }
    let live = std::sync::Arc::new(std::sync::Mutex::new(CodeExecLive {
        started_at: std::time::Instant::now(),
        finished_at: None,
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
        done: false,
    }));
    tab_state.app.code_exec_live = Some(live.clone());
    tab_state.app.code_exec_result_ready = false;
    tab_state.app.code_exec_finished_output = None;
    tab_state.app.code_exec_hover = None;
    tab_state.app.code_exec_reason_target = None;
    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
    tab_state.app.code_exec_scroll = 0;
    tab_state.app.code_exec_stdout_scroll = 0;
    tab_state.app.code_exec_stderr_scroll = 0;
    tab_state.app.code_exec_run_id = None;
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    tab_state.app.code_exec_cancel = Some(cancel.clone());
    let run_id = format!(
        "run-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    tab_state.app.code_exec_run_id = Some(run_id.clone());
    let exec_code = inject_requirements(&pending.code);
    pending.exec_code = Some(exec_code.clone());
    if let Some(current) = tab_state.app.pending_code_exec.as_mut() {
        current.exec_code = Some(exec_code);
    }
    let container_id = match ensure_container(&mut tab_state.app.code_exec_container_id) {
        Ok(id) => id,
        Err(err) => {
            if let Ok(mut live) = live.lock() {
                live.stderr = err;
                live.exit_code = Some(-1);
                live.done = true;
                live.finished_at = Some(std::time::Instant::now());
            }
            return;
        }
    };
    if pending.language == "python" {
        std::thread::spawn(move || {
            let code = pending.exec_code.as_deref().unwrap_or(&pending.code);
            if let Err(err) = run_python_in_container_stream(
                &container_id,
                &run_id,
                code,
                live.clone(),
                cancel,
            ) {
                if let Ok(mut live) = live.lock() {
                    live.stderr.push_str(&format!("{err}\n"));
                    live.exit_code = Some(-1);
                    live.done = true;
                    live.finished_at = Some(std::time::Instant::now());
                }
            }
        });
    } else if let Ok(mut live) = live.lock() {
        live.stderr = format!("不支持的语言：{}", pending.language);
        live.exit_code = Some(-1);
        live.done = true;
        live.finished_at = Some(std::time::Instant::now());
    }
}

pub(crate) fn handle_code_exec_deny(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some(pending) = tab_state.app.pending_code_exec.take() else {
        let idx = tab_state.app.messages.len();
        tab_state.app.messages.push(Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "没有待审批的代码执行请求。".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        tab_state.app.dirty_indices.push(idx);
        return;
    };
    tab_state.app.code_exec_live = None;
    tab_state.app.code_exec_result_ready = false;
    tab_state.app.code_exec_finished_output = None;
    tab_state.app.code_exec_cancel = None;
    tab_state.app.code_exec_hover = None;
    tab_state.app.code_exec_scroll = 0;
    tab_state.app.code_exec_stdout_scroll = 0;
    tab_state.app.code_exec_stderr_scroll = 0;
    let reason = take_code_exec_reason(tab_state, CodeExecReasonTarget::Deny)
        .unwrap_or_else(|| "用户取消".to_string());
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content: format!(r#"{{"error":"用户拒绝执行","reason":"{}"}}"#, escape_json_string(&reason)),
        tool_call_id: Some(pending.call_id),
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
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
        args.log_requests.clone(),
        tab_state.app.log_session_id.clone(),
    );
}
