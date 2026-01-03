use crate::types::{Message, ToolCall, Usage};
use std::sync::mpsc::Sender;
use std::sync::{Arc, atomic::AtomicBool};

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

pub struct LlmStreamRequestParams {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub prompts_dir: String,
    pub enable_web_search: bool,
    pub enable_code_exec: bool,
    pub enable_read_file: bool,
    pub enable_read_code: bool,
    pub enable_modify_file: bool,
    pub log_dir: Option<String>,
    pub log_session_id: String,
    pub message_index: usize,
    pub cancel: Arc<AtomicBool>,
    pub tx: Sender<UiEvent>,
    pub tab: usize,
    pub request_id: u64,
}
