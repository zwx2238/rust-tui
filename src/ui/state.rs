use crate::types::Message;
use tui_textarea::TextArea;
use std::time::Instant;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Focus {
    Chat,
    Input,
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
    pub busy_since: Option<Instant>,
    pub pending_assistant: Option<usize>,
    pub pending_reasoning: Option<usize>,
    pub stream_buffer: String,
    pub assistant_stats: Option<(usize, String)>,
    pub scrollbar_dragging: bool,
    pub chat_selecting: bool,
    pub chat_selection: Option<crate::ui::selection::Selection>,
    pub input_selecting: bool,
    pub dirty_indices: Vec<usize>,
    pub cache_shift: Option<usize>,
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
            input: TextArea::default(),
            input_view_top_row: 0,
            messages,
            scroll: 0,
            follow: true,
            focus: Focus::Input,
            busy: false,
            pending_send: None,
            busy_since: None,
            pending_assistant: None,
            pending_reasoning: None,
            stream_buffer: String::new(),
            assistant_stats: None,
            scrollbar_dragging: false,
            chat_selecting: false,
            chat_selection: None,
            input_selecting: false,
            dirty_indices: Vec::new(),
            cache_shift: None,
        }
    }
}
