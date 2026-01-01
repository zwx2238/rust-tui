use crate::types::{Message, ROLE_SYSTEM};
use std::collections::BTreeMap;
use std::sync::{
    Arc,
    Mutex,
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
    ApproveCodeExec,
    DenyCodeExec,
    ExitCodeExec,
    StopCodeExec,
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

#[derive(Clone)]
pub struct Notice {
    pub text: String,
    pub expires_at: Instant,
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
    pub message_layouts: Vec<crate::render::MessageLayout>,
    pub nav_mode: bool,
    pub tavily_api_key: String,
    pub prompts_dir: String,
    pub pending_code_exec: Option<PendingCodeExec>,
    pub code_exec_scroll: usize,
    pub code_exec_stdout_scroll: usize,
    pub code_exec_stderr_scroll: usize,
    pub code_exec_live: Option<Arc<Mutex<CodeExecLive>>>,
    pub code_exec_result_ready: bool,
    pub code_exec_finished_output: Option<String>,
    pub code_exec_cancel: Option<Arc<AtomicBool>>,
    pub code_exec_hover: Option<CodeExecHover>,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub dirty_indices: Vec<usize>,
    pub cache_shift: Option<usize>,
    pub notice: Option<Notice>,
}

#[derive(Clone, Debug)]
pub struct PendingCodeExec {
    pub call_id: String,
    pub language: String,
    pub code: String,
}

#[derive(Clone, Debug)]
pub struct CodeExecLive {
    pub started_at: Instant,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub done: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CodeExecHover {
    Approve,
    Deny,
    Stop,
    Exit,
}

impl App {
    pub fn new(system_prompt: &str, default_model: &str, default_prompt: &str) -> Self {
        let mut messages = Vec::new();
        if !system_prompt.trim().is_empty() {
            messages.push(Message {
                role: ROLE_SYSTEM.to_string(),
                content: system_prompt.to_string(),
                tool_call_id: None,
                tool_calls: None,
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
            message_layouts: Vec::new(),
            nav_mode: false,
            tavily_api_key: String::new(),
            prompts_dir: String::new(),
            pending_code_exec: None,
            code_exec_scroll: 0,
            code_exec_stdout_scroll: 0,
            code_exec_stderr_scroll: 0,
            code_exec_live: None,
            code_exec_result_ready: false,
            code_exec_finished_output: None,
            code_exec_cancel: None,
            code_exec_hover: None,
            total_prompt_tokens: 0,
            total_completion_tokens: 0,
            total_tokens: 0,
            dirty_indices: Vec::new(),
            cache_shift: None,
            notice: None,
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
                    tool_call_id: None,
                    tool_calls: None,
                },
            );
            self.cache_shift = Some(0);
        }
    }
}
