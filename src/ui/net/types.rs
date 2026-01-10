use crate::types::Message;
use std::sync::mpsc::Sender;
use std::sync::{Arc, atomic::AtomicBool};

pub struct LlmStreamRequestParams {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: Option<u64>,
    pub messages: Vec<Message>,
    pub prompts_dir: String,
    pub show_reasoning: bool,
    pub enable_web_search: bool,
    pub enable_code_exec: bool,
    pub enable_read_file: bool,
    pub enable_read_code: bool,
    pub enable_modify_file: bool,
    pub enable_ask_questions: bool,
    pub log_dir: Option<String>,
    pub log_session_id: String,
    pub message_index: usize,
    pub cancel: Arc<AtomicBool>,
    pub tx: Sender<crate::ui::events::RuntimeEvent>,
    pub tab: String,
    pub request_id: u64,
}
