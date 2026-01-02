use crate::llm::rig::{RigOutcome, prepare_rig_context, rig_complete};
use crate::types::{Message, ToolCall, ToolFunctionCall, Usage};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::runtime::Runtime;

pub enum LlmEvent {
    Chunk(String),
    Error(String),
    Done {
        usage: Option<Usage>,
    },
    ToolCalls {
        calls: Vec<ToolCall>,
        usage: Option<Usage>,
    },
}

pub struct UiEvent {
    pub tab: usize,
    pub request_id: u64,
    pub event: LlmEvent,
}

pub fn request_llm_stream(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[Message],
    prompts_dir: &str,
    enable_web_search: bool,
    enable_code_exec: bool,
    enable_read_file: bool,
    enable_read_code: bool,
    enable_modify_file: bool,
    log_dir: Option<String>,
    log_session_id: String,
    message_index: usize,
    cancel: Arc<AtomicBool>,
    tx: Sender<UiEvent>,
    tab: usize,
    request_id: u64,
) {
    let messages = messages.to_vec();
    let prompts_dir = prompts_dir.to_string();
    let log_dir = log_dir.clone();
    let log_session_id = log_session_id.clone();
    let base_url = base_url.to_string();
    let api_key = api_key.to_string();
    let model = model.to_string();
    let enabled = build_enabled_tools(
        enable_web_search,
        enable_code_exec,
        enable_read_file,
        enable_read_code,
        enable_modify_file,
    );
    let rt = Runtime::new();
    if rt.is_err() {
        let _ = tx.send(UiEvent {
            tab,
            request_id,
            event: LlmEvent::Error("初始化 Tokio 失败".to_string()),
        });
        return;
    }
    let rt = rt.unwrap();
    let result = rt.block_on(async {
        let (ctx, _templates) = prepare_rig_context(&messages, &prompts_dir, &enabled)?;
        if let Some(dir) = log_dir.as_deref() {
            let _ = write_request_log(
                dir,
                &log_session_id,
                tab,
                message_index,
                &base_url,
                &model,
                &ctx,
            );
        }
        rig_complete(&base_url, &api_key, &model, ctx).await
    });
    match result {
        Ok(RigOutcome::Message { content, usage }) => {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            if let Some(dir) = log_dir.as_deref() {
                let _ = write_response_log(dir, &log_session_id, tab, message_index, &content);
            }
            stream_chunks(&content, &cancel, &tx, tab, request_id);
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::Done { usage },
            });
        }
        Ok(RigOutcome::ToolCall { name, args, usage }) => {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            if let Some(dir) = log_dir.as_deref() {
                let payload = format!(
                    "tool_call: {name}\nargs: {}",
                    serde_json::to_string_pretty(&args).unwrap_or_default()
                );
                let _ = write_response_log(dir, &log_session_id, tab, message_index, &payload);
            }
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::Chunk(format!("调用工具：{name}\n")),
            });
            let call = ToolCall {
                id: format!("rig-{}-{}", tab, request_id),
                kind: "function".to_string(),
                function: ToolFunctionCall {
                    name,
                    arguments: serde_json::to_string(&args).unwrap_or_default(),
                },
            };
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::ToolCalls {
                    calls: vec![call],
                    usage,
                },
            });
        }
        Err(e) => {
            if let Some(dir) = log_dir.as_deref() {
                let payload = format!("error: {e}");
                let _ = write_response_log(dir, &log_session_id, tab, message_index, &payload);
            }
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::Error(e),
            });
        }
    }
}

fn build_enabled_tools(
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

fn write_request_log(
    dir: &str,
    session_id: &str,
    tab: usize,
    message_index: usize,
    base_url: &str,
    model: &str,
    ctx: &crate::llm::rig::RigRequestContext,
) -> std::io::Result<()> {
    let path = build_log_path(dir, session_id, tab, message_index, "input.txt")?;
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
    for msg in &ctx.history {
        out.push('[');
        out.push_str(&msg.role);
        out.push_str("]\n");
        out.push_str(&msg.content);
        out.push('\n');
    }
    out.push_str("--- prompt ---\n");
    out.push_str(&ctx.prompt);
    out.push('\n');
    fs::write(path, out)
}

fn write_response_log(
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

fn stream_chunks(
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
            let chunk: String = buf.drain(..).collect();
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::Chunk(chunk),
            });
            std::thread::sleep(Duration::from_millis(8));
        }
    }
    if !buf.is_empty() {
        let chunk: String = buf.drain(..).collect();
        let _ = tx.send(UiEvent {
            tab,
            request_id,
            event: LlmEvent::Chunk(chunk),
        });
    }
}
