use crate::llm::rig::{RigOutcome, prepare_rig_context, rig_complete};
use crate::types::{Message, ToolCall, ToolFunctionCall, Usage};
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
    Done { usage: Option<Usage> },
    ToolCalls { calls: Vec<ToolCall>, usage: Option<Usage> },
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
    cancel: Arc<AtomicBool>,
    tx: Sender<UiEvent>,
    tab: usize,
    request_id: u64,
) {
    let messages = messages.to_vec();
    let prompts_dir = prompts_dir.to_string();
    let base_url = base_url.to_string();
    let api_key = api_key.to_string();
    let model = model.to_string();
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
        let (ctx, _templates) = prepare_rig_context(&messages, &prompts_dir)?;
        rig_complete(&base_url, &api_key, &model, ctx).await
    });
    match result {
        Ok(RigOutcome::Message(content)) => {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            stream_chunks(&content, &cancel, &tx, tab, request_id);
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::Done { usage: None },
            });
        }
        Ok(RigOutcome::ToolCall { name, args }) => {
            if cancel.load(Ordering::Relaxed) {
                return;
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
                event: LlmEvent::ToolCalls { calls: vec![call], usage: None },
            });
        }
        Err(e) => {
            let _ = tx.send(UiEvent {
                tab,
                request_id,
                event: LlmEvent::Error(e),
            });
        }
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
