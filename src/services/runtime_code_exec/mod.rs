mod exec;
mod helpers;
mod pending;

use crate::args::Args;
use crate::services::code_exec_container::ensure_container_cached;
use crate::ui::events::RuntimeEvent;
use crate::services::runtime_code_exec_helpers::inject_requirements;
use crate::services::runtime_code_exec_output::{escape_json_string, take_code_exec_reason};
use crate::ui::runtime_helpers::TabState;
use crate::services::runtime_requests::start_followup_request;
use crate::ui::state::{CodeExecReasonTarget, PendingCodeExec};
use crate::services::tools::{parse_bash_exec_args, parse_code_exec_args};
use std::sync::mpsc;
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
    helpers::reset_code_exec_ui(&mut tab_state.app);
    Ok(())
}

pub(crate) fn handle_bash_exec_request(
    tab_state: &mut TabState,
    call: &crate::types::ToolCall,
) -> Result<(), String> {
    if tab_state.app.pending_code_exec.is_some() {
        return Err("已有待审批的代码执行请求".to_string());
    }
    let request = parse_bash_exec_args(&call.function.arguments)?;
    tab_state.app.pending_code_exec = Some(PendingCodeExec {
        call_id: call.id.clone(),
        language: request.language,
        code: request.code,
        exec_code: None,
        requested_at: Instant::now(),
        stop_reason: None,
    });
    helpers::reset_code_exec_ui(&mut tab_state.app);
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
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    let Some((pending, content)) = helpers::take_pending_and_output(&mut tab_state.app) else {
        return;
    };
    helpers::push_tool_message(&mut tab_state.app, content, pending.call_id);
    helpers::reset_code_exec_after_exit(&mut tab_state.app);
    start_followup(tab_state, registry, args, tx);
}

pub(crate) fn handle_code_exec_approve(
    tab_state: &mut TabState,
    _tab_id: usize,
    _registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    _tx: &mpsc::Sender<RuntimeEvent>,
) {
    let Some(mut pending) = pending::clone_pending_or_notify(&mut tab_state.app) else {
        return;
    };
    if tab_state.app.code_exec_live.is_some() {
        return;
    }
    let (live, cancel, run_id) = init_exec_state(tab_state);
    let exec_code = build_exec_code(&pending);
    helpers::store_exec_code(&mut tab_state.app, &mut pending, exec_code);
    let workspace = match crate::services::workspace::resolve_workspace(args) {
        Ok(val) => val,
        Err(err) => {
            helpers::mark_exec_error(&live, err);
            return;
        }
    };
    spawn_exec_thread(workspace, pending, live, cancel, run_id);
}

fn init_exec_state(
    tab_state: &mut TabState,
) -> (
    std::sync::Arc<std::sync::Mutex<crate::ui::state::CodeExecLive>>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
    String,
) {
    let live = helpers::init_code_exec_live(&mut tab_state.app);
    let cancel = helpers::init_cancel_flag(&mut tab_state.app);
    let run_id = helpers::init_run_id(&mut tab_state.app);
    (live, cancel, run_id)
}

fn build_exec_code(pending: &PendingCodeExec) -> String {
    if pending.language == "python" {
        inject_requirements(&pending.code)
    } else {
        pending.code.clone()
    }
}

fn spawn_exec_thread(
    workspace: crate::services::workspace::WorkspaceConfig,
    pending: PendingCodeExec,
    live: std::sync::Arc<std::sync::Mutex<crate::ui::state::CodeExecLive>>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    run_id: String,
) {
    std::thread::spawn(move || {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        let container_id = match ensure_container_cached(&workspace) {
            Ok(id) => id,
            Err(err) => {
                helpers::mark_exec_error(&live, err);
                return;
            }
        };
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        exec::spawn_exec(container_id, run_id, pending, live, cancel);
    });
}

pub(crate) fn handle_code_exec_deny(
    tab_state: &mut TabState,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    let Some(pending) = pending::take_pending_or_notify(&mut tab_state.app) else {
        return;
    };
    helpers::reset_code_exec_after_deny(&mut tab_state.app);
    let reason = take_code_exec_reason(tab_state, CodeExecReasonTarget::Deny)
        .unwrap_or_else(|| "用户取消".to_string());
    let content = format!(
        r#"{{"error":"用户拒绝执行","reason":"{}"}}"#,
        escape_json_string(&reason)
    );
    helpers::push_tool_message(&mut tab_state.app, content, pending.call_id);
    start_followup(tab_state, registry, args, tx);
}

fn start_followup(
    tab_state: &mut TabState,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<RuntimeEvent>,
) {
    let model = registry
        .get(&tab_state.app.model_key)
        .unwrap_or_else(|| registry.get(&registry.default_key).expect("model"));
    let log_session_id = tab_state.app.log_session_id.clone();
    start_followup_request(crate::services::runtime_requests::StartFollowupRequestParams {
        tab_state,
        base_url: &model.base_url,
        api_key: &model.api_key,
        model: &model.model,
        max_tokens: model.max_tokens,
        show_reasoning: args.show_reasoning,
        tx,
        enable_web_search: args.web_search_enabled(),
        enable_code_exec: args.code_exec_enabled(),
        enable_read_file: args.read_file_enabled(),
        enable_read_code: args.read_code_enabled(),
        enable_modify_file: args.modify_file_enabled(),
        enable_ask_questions: args.ask_questions_enabled(),
        log_requests: args.log_requests.clone(),
        log_session_id,
    });
}
