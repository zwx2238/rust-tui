use crate::ui::runtime_code_exec_helpers::filter_pip_output;
use crate::ui::state::{CodeExecLive, CodeExecReasonTarget, PendingCodeExec};
use crate::ui::runtime_helpers::TabState;

pub(crate) fn build_code_exec_tool_output(
    pending: &PendingCodeExec,
    live: &CodeExecLive,
) -> String {
    let stdout_filtered = filter_pip_output(&live.stdout, live.exit_code);
    let stdout_empty = stdout_filtered.trim().is_empty();
    let stderr_empty = live.stderr.trim().is_empty();
    let mut text = String::new();
    text.push_str("[code_exec]\n");
    text.push_str(&format!("language: {}\n", pending.language));
    text.push_str("code:\n");
    if pending.code.trim().is_empty() {
        text.push_str("(空)\n");
    } else {
        text.push_str("```python\n");
        text.push_str(&pending.code);
        if !pending.code.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("```\n");
    }
    if let Some(code) = live.exit_code {
        text.push_str(&format!("exit_code: {}\n", code));
    } else {
        text.push_str("exit_code: (执行中)\n");
    }
    text.push_str("stdout:\n");
    if stdout_empty {
        text.push_str("(空)\n");
    } else {
        text.push_str("```text\n");
        text.push_str(&stdout_filtered);
        if !stdout_filtered.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("```\n");
    }
    text.push_str("stderr:\n");
    if stderr_empty {
        text.push_str("(空)\n");
    } else {
        text.push_str("```text\n");
        text.push_str(&live.stderr);
        if !live.stderr.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("```\n");
    }
    if live.done
        && live.exit_code == Some(0)
        && stdout_empty
        && stderr_empty
    {
        text.push_str("note: 程序正常执行但没有输出。\n");
    }
    if let Some(reason) = pending.stop_reason.as_ref() {
        text.push_str(&format!("stop_reason: {}\n", reason));
    }
    text
}

pub(crate) fn take_code_exec_reason(
    tab_state: &mut TabState,
    target: CodeExecReasonTarget,
) -> Option<String> {
    if tab_state.app.code_exec_reason_target != Some(target) {
        return None;
    }
    let reason = tab_state.app.code_exec_reason_input.lines().join("\n");
    tab_state.app.code_exec_reason_target = None;
    tab_state.app.code_exec_reason_input = tui_textarea::TextArea::default();
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn escape_json_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
