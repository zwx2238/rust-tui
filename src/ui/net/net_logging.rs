use crate::llm::rig::RigRequestContext;
use crate::ui::events::{LlmEvent, RuntimeEvent, send_llm};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

pub(super) fn build_enabled_tools(
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    enable_ask_questions: bool,
) -> Vec<&'static str> {
    let mut out = Vec::new();
    if enable_web_search {
        out.push("web_search");
    }
    if enable_code_exec {
        out.push("code_exec");
        out.push("bash_exec");
    }
    if enable_read_file {
        out.push("read_file");
        out.push("list_dir");
    }
    if enable_read_code {
        out.push("read_code");
    }
    if enable_modify_file {
        out.push("modify_file");
    }
    if enable_ask_questions {
        out.push("ask_questions");
    }
    out
}

pub(super) fn write_request_log(
    dir: &str,
    session_id: &str,
    tab: &str,
    message_index: usize,
    base_url: &str,
    model: &str,
    ctx: &RigRequestContext,
) -> std::io::Result<()> {
    let path = build_log_path(dir, session_id, tab, message_index, "input.txt")?;
    let content = format_request_log(base_url, model, ctx);
    fs::write(path, content)
}

fn format_request_log(base_url: &str, model: &str, ctx: &RigRequestContext) -> String {
    let mut out = String::new();
    out.push_str("base_url: ");
    out.push_str(base_url);
    out.push('\n');
    out.push_str("model: ");
    out.push_str(model);
    out.push('\n');
    out.push_str("--- preamble ---\n");
    out.push_str(&ctx.preamble);
    out.push('\n');
    out.push_str("--- history ---\n");
    append_history_log(&mut out, &ctx.history);
    out.push_str("--- prompt ---\n");
    out.push_str(&ctx.prompt);
    out.push('\n');
    out
}

fn append_history_log(out: &mut String, history: &[rig::completion::Message]) {
    for msg in history {
        let (role, content) = message_log_entry(msg);
        out.push('[');
        out.push_str(role);
        out.push_str("]\n");
        if !content.is_empty() {
            out.push_str(&content);
            out.push('\n');
        }
    }
}

fn message_log_entry(msg: &rig::completion::Message) -> (&'static str, String) {
    match msg {
        rig::completion::Message::User { content } => ("user", user_content_text(content)),
        rig::completion::Message::Assistant { content, .. } => {
            ("assistant", assistant_content_text(content))
        }
    }
}

fn user_content_text(content: &rig::OneOrMany<rig::completion::message::UserContent>) -> String {
    let mut parts = Vec::new();
    for item in content.iter() {
        match item {
            rig::completion::message::UserContent::Text(text) => parts.push(text.text.clone()),
            rig::completion::message::UserContent::ToolResult(result) => {
                parts.push(tool_result_text(result));
            }
            _ => parts.push("[非文本内容]".to_string()),
        }
    }
    parts.join("\n")
}

fn tool_result_text(result: &rig::completion::message::ToolResult) -> String {
    let mut parts = Vec::new();
    for item in result.content.iter() {
        match item {
            rig::completion::message::ToolResultContent::Text(text) => {
                parts.push(text.text.clone());
            }
            _ => parts.push("[工具输出非文本]".to_string()),
        }
    }
    parts.join("\n")
}

fn assistant_content_text(content: &rig::OneOrMany<rig::completion::AssistantContent>) -> String {
    let mut parts = Vec::new();
    for item in content.iter() {
        match item {
            rig::completion::AssistantContent::Text(text) => parts.push(text.text.clone()),
            _ => parts.push("[非文本内容]".to_string()),
        }
    }
    parts.join("\n")
}

pub(super) fn write_response_log(
    dir: &str,
    session_id: &str,
    tab: &str,
    message_index: usize,
    content: &str,
) -> std::io::Result<()> {
    let path = build_log_path(dir, session_id, tab, message_index, "output.txt")?;
    fs::write(path, content)
}

fn build_log_path(
    dir: &str,
    session_id: &str,
    tab: &str,
    message_index: usize,
    suffix: &str,
) -> std::io::Result<PathBuf> {
    let dir = Path::new(dir);
    fs::create_dir_all(dir)?;
    let session = sanitize_log_part(session_id);
    let tab = sanitize_log_part(tab);
    let msg = message_index + 1;
    let filename = format!("{session}_tab{tab}_msg{msg}_{suffix}");
    Ok(dir.join(filename))
}

fn sanitize_log_part(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "session".to_string()
    } else {
        out
    }
}

pub(super) fn stream_chunks(
    text: &str,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<RuntimeEvent>,
    tab: &str,
    request_id: u64,
) {
    let mut buf: Vec<char> = Vec::new();
    for ch in text.chars() {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        buf.push(ch);
        if buf.len() >= 32 {
            send_chunk(&mut buf, tx, tab, request_id, true);
        }
    }
    if !buf.is_empty() {
        send_chunk(&mut buf, tx, tab, request_id, false);
    }
}

fn send_chunk(
    buf: &mut Vec<char>,
    tx: &Sender<RuntimeEvent>,
    tab: &str,
    request_id: u64,
    pause: bool,
) {
    let chunk: String = buf.drain(..).collect();
    send_llm(tx, tab.to_string(), request_id, LlmEvent::Chunk(chunk));
    if pause {
        std::thread::sleep(Duration::from_millis(8));
    }
}
