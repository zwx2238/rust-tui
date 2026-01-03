use super::{LlmEvent, UiEvent};
use crate::llm::rig::RigRequestContext;
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
) -> Vec<&'static str> {
    let mut out = Vec::new();
    if enable_web_search {
        out.push("web_search");
    }
    if enable_code_exec {
        out.push("code_exec");
    }
    if enable_read_file {
        out.push("read_file");
    }
    if enable_read_code {
        out.push("read_code");
    }
    if enable_modify_file {
        out.push("modify_file");
    }
    out
}

pub(super) fn write_request_log(
    dir: &str,
    session_id: &str,
    tab: usize,
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
    tab: usize,
    message_index: usize,
    content: &str,
) -> std::io::Result<()> {
    let path = build_log_path(dir, session_id, tab, message_index, "output.txt")?;
    fs::write(path, content)
}

fn build_log_path(
    dir: &str,
    session_id: &str,
    tab: usize,
    message_index: usize,
    suffix: &str,
) -> std::io::Result<PathBuf> {
    let dir = Path::new(dir);
    fs::create_dir_all(dir)?;
    let session = sanitize_log_part(session_id);
    let tab = tab + 1;
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
    tx: &Sender<UiEvent>,
    tab: usize,
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

fn send_chunk(buf: &mut Vec<char>, tx: &Sender<UiEvent>, tab: usize, request_id: u64, pause: bool) {
    let chunk: String = buf.drain(..).collect();
    let _ = tx.send(UiEvent {
        tab,
        request_id,
        event: LlmEvent::Chunk(chunk),
    });
    if pause {
        std::thread::sleep(Duration::from_millis(8));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_enabled_tools, build_log_path, sanitize_log_part, stream_chunks, write_request_log,
        write_response_log,
    };
    use crate::llm::rig::RigRequestContext;
    use crate::test_support::{env_lock, restore_env, set_env};
    use rig::completion::Message;
    use std::fs;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("deepchat_net_{name}_{ts}"));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn rig_context() -> RigRequestContext {
        RigRequestContext {
            preamble: "PRE".to_string(),
            history: vec![Message::user("hi")],
            prompt: "PROMPT".to_string(),
            tools: Vec::new(),
        }
    }

    fn write_logs(dir: &std::path::Path, ctx: &RigRequestContext) {
        write_request_log(
            dir.to_string_lossy().as_ref(),
            "sess",
            0,
            0,
            "http://example.com",
            "model",
            ctx,
        )
        .unwrap();
        write_response_log(dir.to_string_lossy().as_ref(), "sess", 0, 0, "output").unwrap();
    }

    #[test]
    fn build_enabled_tools_collects_flags() {
        let tools = build_enabled_tools(true, false, true, false, true);
        assert!(tools.contains(&"web_search"));
        assert!(tools.contains(&"read_file"));
        assert!(tools.contains(&"modify_file"));
        assert!(!tools.contains(&"code_exec"));
    }

    #[test]
    fn sanitize_log_part_replaces_invalid() {
        assert_eq!(sanitize_log_part("a/b c"), "a_b_c");
        assert_eq!(sanitize_log_part(""), "session");
    }

    #[test]
    fn build_log_path_creates_dir_and_sanitizes() {
        let dir = temp_dir("path");
        let path =
            build_log_path(dir.to_string_lossy().as_ref(), "a/b", 0, 1, "input.txt").unwrap();
        assert!(path.exists() || path.parent().is_some());
        assert!(path.to_string_lossy().contains("a_b_tab1_msg2_input.txt"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_request_and_response_log_outputs_files() {
        let _guard = env_lock().lock().unwrap();
        let dir = temp_dir("log");
        let ctx = rig_context();
        write_logs(&dir, &ctx);
        let input = fs::read_to_string(dir.join("sess_tab1_msg1_input.txt")).unwrap();
        assert!(input.contains("base_url: http://example.com"));
        assert!(input.contains("--- preamble ---"));
        let output = fs::read_to_string(dir.join("sess_tab1_msg1_output.txt")).unwrap();
        assert_eq!(output, "output");
        let _ = fs::remove_dir_all(&dir);
        let _ = set_env("DUMMY", "1");
        restore_env("DUMMY", None);
    }

    #[test]
    fn stream_chunks_sends_events() {
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        stream_chunks("hello", &cancel, &tx, 1, 2);
        let msg = rx.recv_timeout(Duration::from_millis(50)).unwrap();
        match msg.event {
            super::LlmEvent::Chunk(text) => assert!(!text.is_empty()),
            _ => panic!("unexpected event"),
        }
        cancel.store(true, Ordering::Relaxed);
    }
}
