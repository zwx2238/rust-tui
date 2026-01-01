use crate::types::{Message, ROLE_SYSTEM};
use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;
use tui_textarea::TextArea;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Focus {
    Chat,
    Input,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PendingCommand {
    SaveSession,
}

#[derive(Clone)]
pub struct RequestHandle {
    pub id: u64,
    pub cancel: Arc<AtomicBool>,
}

impl RequestHandle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

pub struct App {
    pub input: TextArea<'static>,
    pub input_view_top_row: u16,
    pub messages: Vec<Message>,
    pub scroll: u16,
    pub follow: bool,
    pub focus: Focus,
    pub busy: bool,
    pub pending_send: Option<String>,
    pub pending_command: Option<PendingCommand>,
    pub active_request: Option<RequestHandle>,
    pub next_request_id: u64,
    pub busy_since: Option<Instant>,
    pub pending_assistant: Option<usize>,
    pub pending_reasoning: Option<usize>,
    pub stream_buffer: String,
    pub assistant_stats: BTreeMap<usize, String>,
    pub scrollbar_dragging: bool,
    pub chat_selecting: bool,
    pub chat_selection: Option<crate::ui::selection::Selection>,
    pub input_selecting: bool,
    pub model_key: String,
    pub prompt_key: String,
    pub dirty_indices: Vec<usize>,
    pub cache_shift: Option<usize>,
}

impl App {
    pub fn new(system_prompt: &str, default_model: &str, default_prompt: &str) -> Self {
        let mut messages = Vec::new();
        if !system_prompt.trim().is_empty() {
            messages.push(Message {
                role: ROLE_SYSTEM.to_string(),
                content: system_prompt.to_string(),
            });
        }
        Self {
            input: TextArea::default(),
            input_view_top_row: 0,
            messages,
            scroll: 0,
            follow: true,
            focus: Focus::Input,
            busy: false,
            pending_send: None,
            pending_command: None,
            active_request: None,
            next_request_id: 1,
            busy_since: None,
            pending_assistant: None,
            pending_reasoning: None,
            stream_buffer: String::new(),
            assistant_stats: BTreeMap::new(),
            scrollbar_dragging: false,
            chat_selecting: false,
            chat_selection: None,
            input_selecting: false,
            model_key: default_model.to_string(),
            prompt_key: default_prompt.to_string(),
            dirty_indices: Vec::new(),
            cache_shift: None,
        }
    }

    pub fn set_system_prompt(&mut self, key: &str, content: &str) {
        self.prompt_key = key.to_string();
        if let Some(msg) = self.messages.iter_mut().find(|m| m.role == ROLE_SYSTEM) {
            msg.content = content.to_string();
            return;
        }
        if !content.trim().is_empty() {
            self.messages.insert(
                0,
                Message {
                    role: ROLE_SYSTEM.to_string(),
                    content: content.to_string(),
                },
            );
            self.cache_shift = Some(0);
        }
    }
}
