use crate::ui::runtime_code_exec_helpers::filter_pip_output;
use crate::ui::runtime_helpers::TabState;
use crate::ui::state::{CodeExecLive, CodeExecReasonTarget, PendingCodeExec};

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
    append_code_block(&mut text, &pending.code, &pending.language);
    append_exit_code(&mut text, live.exit_code);
    append_output_block(&mut text, "stdout", &stdout_filtered, stdout_empty);
    append_output_block(&mut text, "stderr", &live.stderr, stderr_empty);
    append_empty_note(&mut text, live, stdout_empty, stderr_empty);
    append_stop_reason(&mut text, pending);
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

fn append_code_block(out: &mut String, code: &str, language: &str) {
    out.push_str("code:\n");
    if code.trim().is_empty() {
        out.push_str("(空)\n");
        return;
    }
    let lang = if language.trim().is_empty() {
        "text"
    } else {
        language
    };
    out.push_str("```");
    out.push_str(lang);
    out.push('\n');
    out.push_str(code);
    if !code.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n");
}

fn append_exit_code(out: &mut String, exit_code: Option<i32>) {
    if let Some(code) = exit_code {
        out.push_str(&format!("exit_code: {}\n", code));
    } else {
        out.push_str("exit_code: (执行中)\n");
    }
}

fn append_output_block(out: &mut String, label: &str, content: &str, empty: bool) {
    out.push_str(label);
    out.push_str(":\n");
    if empty {
        out.push_str("(空)\n");
        return;
    }
    out.push_str("```text\n");
    out.push_str(content);
    if !content.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n");
}

fn append_empty_note(
    out: &mut String,
    live: &CodeExecLive,
    stdout_empty: bool,
    stderr_empty: bool,
) {
    if live.done && live.exit_code == Some(0) && stdout_empty && stderr_empty {
        out.push_str("note: 程序正常执行但没有输出。\n");
    }
}

fn append_stop_reason(out: &mut String, pending: &PendingCodeExec) {
    if let Some(reason) = pending.stop_reason.as_ref() {
        out.push_str(&format!("stop_reason: {}\n", reason));
    }
}
