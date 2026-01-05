use crate::args::Args;
use crate::types::Message;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_followup_request;
use crate::ui::workspace::resolve_workspace;
use crate::ui::code_exec_container::ensure_container_cached;
use crate::ui::state::PendingFilePatch;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc;

#[derive(serde::Deserialize)]
struct PatchArgs {
    diff: String,
    path: Option<String>,
}

pub(crate) fn handle_file_patch_request(
    tab_state: &mut TabState,
    call: &crate::types::ToolCall,
) -> Result<(), String> {
    if tab_state.app.pending_file_patch.is_some() {
        return Err("已有待审批的文件修改请求".to_string());
    }
    let args: PatchArgs = serde_json::from_str(&call.function.arguments)
        .map_err(|e| format!("modify_file 参数解析失败：{e}"))?;
    if args.diff.trim().is_empty() {
        return Err("modify_file 参数 diff 不能为空".to_string());
    }
    let mut diff = args.diff.clone();
    if !diff.ends_with('\n') {
        diff.push('\n');
    }
    let preview = render_diff_preview(&diff);
    tab_state.app.pending_file_patch = Some(PendingFilePatch {
        call_id: call.id.clone(),
        path: args.path,
        diff,
        preview,
    });
    tab_state.app.file_patch_scroll = 0;
    tab_state.app.file_patch_hover = None;
    tab_state.app.file_patch_selecting = false;
    tab_state.app.file_patch_selection = None;
    Ok(())
}

pub(crate) fn handle_file_patch_apply(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some(pending) = tab_state.app.pending_file_patch.take() else {
        return;
    };
    let message = build_apply_message(&pending, apply_patch(&pending.diff, args));
    push_tool_message(&mut tab_state.app, message, pending.call_id);
    reset_patch_ui(&mut tab_state.app);
    start_followup(tab_state, tab_id, registry, args, tx);
}

pub(crate) fn handle_file_patch_cancel(
    tab_state: &mut TabState,
    tab_id: usize,
    registry: &crate::model_registry::ModelRegistry,
    args: &Args,
    tx: &mpsc::Sender<UiEvent>,
) {
    let Some(pending) = tab_state.app.pending_file_patch.take() else {
        return;
    };
    push_tool_message(
        &mut tab_state.app,
        r#"{"error":"用户取消"}"#.to_string(),
        pending.call_id,
    );
    reset_patch_ui(&mut tab_state.app);
    start_followup(tab_state, tab_id, registry, args, tx);
}

fn build_apply_message(pending: &PendingFilePatch, result: Result<(), String>) -> String {
    match result {
        Ok(()) => format!(
            r#"{{"ok":true,"message":"已应用补丁{}"}}"#,
            pending
                .path
                .as_ref()
                .map(|p| format!(" ({p})"))
                .unwrap_or_default()
        ),
        Err(err) => format!(r#"{{"error":"{}"}}"#, escape_json_string(&err)),
    }
}

fn push_tool_message(app: &mut crate::ui::state::App, content: String, call_id: String) {
    let idx = app.messages.len();
    app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content,
        tool_call_id: Some(call_id),
        tool_calls: None,
    });
    app.dirty_indices.push(idx);
}

fn reset_patch_ui(app: &mut crate::ui::state::App) {
    app.file_patch_scroll = 0;
    app.file_patch_hover = None;
    app.file_patch_selecting = false;
    app.file_patch_selection = None;
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
    let log_session_id = tab_state.app.log_session_id.clone();
    start_followup_request(crate::ui::runtime_requests::StartFollowupRequestParams {
        tab_state,
        base_url: &model.base_url,
        api_key: &model.api_key,
        model: &model.model,
        _show_reasoning: args.show_reasoning,
        tx,
        tab_id,
        enable_web_search: args.web_search_enabled(),
        enable_code_exec: args.code_exec_enabled(),
        enable_read_file: args.read_file_enabled(),
        enable_read_code: args.read_code_enabled(),
        enable_modify_file: args.modify_file_enabled(),
        log_requests: args.log_requests.clone(),
        log_session_id,
    });
}

fn render_diff_preview(diff: &str) -> String {
    let delta = Command::new("delta")
        .arg("--no-color")
        .arg("--line-numbers")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    if let Ok(mut child) = delta {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(diff.as_bytes());
        }
        if let Ok(output) = child.wait_with_output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if !text.trim().is_empty() {
                return text;
            }
        }
    }
    diff.to_string()
}

fn apply_patch(diff: &str, args: &Args) -> Result<(), String> {
    let workspace = resolve_workspace(args)?;
    let container_id = ensure_container_cached(&workspace)?;
    run_container_patch(&container_id, diff)
}

fn run_container_patch(container_id: &str, diff: &str) -> Result<(), String> {
    let mut cmd = Command::new("docker");
    cmd.arg("exec")
        .arg("-i")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg("cd \"$DEEPCHAT_WORKSPACE\" && git apply --whitespace=nowarn -p1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("应用补丁失败：{e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(diff.as_bytes())
            .map_err(|e| format!("应用补丁失败：{e}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("应用补丁失败：{e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if err.is_empty() {
            "应用补丁失败".to_string()
        } else {
            err
        })
    }
}

fn escape_json_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
