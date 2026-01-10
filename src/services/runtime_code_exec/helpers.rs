use crate::types::Message;
use crate::ui::state::{App, CodeExecLive, PendingCodeExec};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn reset_code_exec_ui(app: &mut App) {
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

pub(super) fn init_code_exec_live(app: &mut App) -> std::sync::Arc<std::sync::Mutex<CodeExecLive>> {
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

pub(super) fn init_cancel_flag(app: &mut App) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    app.code_exec_cancel = Some(cancel.clone());
    cancel
}

pub(super) fn init_run_id(app: &mut App) -> String {
    let run_id = format!(
        "run-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    app.code_exec_run_id = Some(run_id.clone());
    run_id
}

pub(super) fn mark_exec_error(live: &std::sync::Arc<std::sync::Mutex<CodeExecLive>>, err: String) {
    if let Ok(mut live) = live.lock() {
        live.stderr.push_str(&format!("{err}\n"));
        live.exit_code = Some(-1);
        live.done = true;
        live.finished_at = Some(std::time::Instant::now());
    }
}

pub(super) fn mark_unsupported_language(
    live: &std::sync::Arc<std::sync::Mutex<CodeExecLive>>,
    language: &str,
) {
    if let Ok(mut live) = live.lock() {
        live.stderr = format!("不支持的语言：{}", language);
        live.exit_code = Some(-1);
        live.done = true;
        live.finished_at = Some(std::time::Instant::now());
    }
}

pub(super) fn take_pending_and_output(app: &mut App) -> Option<(PendingCodeExec, String)> {
    let pending = app.pending_code_exec.take()?;
    let content = app.code_exec_finished_output.take()?;
    Some((pending, content))
}

pub(super) fn push_tool_message(app: &mut App, content: String, call_id: String) {
    let idx = app.messages.len();
    app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content,
        tool_call_id: Some(call_id),
        tool_calls: None,
    });
    app.dirty_indices.push(idx);
}

pub(super) fn reset_code_exec_after_exit(app: &mut App) {
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

pub(super) fn reset_code_exec_after_deny(app: &mut App) {
    app.code_exec_live = None;
    app.code_exec_result_ready = false;
    app.code_exec_finished_output = None;
    app.code_exec_cancel = None;
    app.code_exec_hover = None;
    app.code_exec_scroll = 0;
    app.code_exec_stdout_scroll = 0;
    app.code_exec_stderr_scroll = 0;
}

pub(super) fn store_exec_code(app: &mut App, pending: &mut PendingCodeExec, exec_code: String) {
    pending.exec_code = Some(exec_code.clone());
    if let Some(current) = app.pending_code_exec.as_mut() {
        current.exec_code = Some(exec_code);
    }
}
