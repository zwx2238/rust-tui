use crate::args::Args;
use crate::types::Message;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_followup_request;
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
    let diff = args.diff.trim().to_string();
    if diff.is_empty() {
        return Err("modify_file 参数 diff 不能为空".to_string());
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
    let apply_result = apply_patch(&pending.diff);
    let idx = tab_state.app.messages.len();
    match apply_result {
        Ok(()) => {
            tab_state.app.messages.push(Message {
                role: crate::types::ROLE_TOOL.to_string(),
                content: format!(
                    r#"{{"ok":true,"message":"已应用补丁{}"}}"#,
                    pending
                        .path
                        .as_ref()
                        .map(|p| format!(" ({p})"))
                        .unwrap_or_default()
                ),
                tool_call_id: Some(pending.call_id),
                tool_calls: None,
            });
        }
        Err(err) => {
            tab_state.app.messages.push(Message {
                role: crate::types::ROLE_TOOL.to_string(),
                content: format!(r#"{{"error":"{}"}}"#, escape_json_string(&err)),
                tool_call_id: Some(pending.call_id),
                tool_calls: None,
            });
        }
    }
    tab_state.app.dirty_indices.push(idx);
    tab_state.app.file_patch_scroll = 0;
    tab_state.app.file_patch_hover = None;
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
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content: r#"{"error":"用户取消"}"#.to_string(),
        tool_call_id: Some(pending.call_id),
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
    tab_state.app.file_patch_scroll = 0;
    tab_state.app.file_patch_hover = None;
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
        if let Ok(output) = child.wait_with_output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                if !text.trim().is_empty() {
                    return text;
                }
            }
        }
    }
    diff.to_string()
}

fn apply_patch(diff: &str) -> Result<(), String> {
    let mut cmd = Command::new("git");
    cmd.arg("apply")
        .arg("--whitespace=nowarn")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
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
