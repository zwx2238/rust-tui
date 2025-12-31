use crate::types::Message;
use std::time::Instant;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Focus {
    Chat,
    Input,
}

pub struct App {
    pub input: String,
    pub cursor: usize,
    pub messages: Vec<Message>,
    pub scroll: u16,
    pub follow: bool,
    pub focus: Focus,
    pub busy: bool,
    pub pending_send: Option<String>,
    pub busy_since: Option<Instant>,
    pub pending_assistant: Option<usize>,
    pub assistant_stats: Option<(usize, String)>,
}

impl App {
    pub fn new(system_prompt: &str) -> Self {
        let mut messages = Vec::new();
        if !system_prompt.trim().is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            });
        }
        Self {
            input: String::new(),
            cursor: 0,
            messages,
            scroll: 0,
            follow: true,
            focus: Focus::Input,
            busy: false,
            pending_send: None,
            busy_since: None,
            pending_assistant: None,
            assistant_stats: None,
        }
    }
}
