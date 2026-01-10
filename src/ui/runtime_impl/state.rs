use crate::types::{Message, ROLE_SYSTEM};
use crate::ui::commands::CommandSuggestion;
use crate::ui::selection_state::SelectionState;
use std::collections::BTreeMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;
use tui_textarea::TextArea;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum Focus {
    Chat,
    #[default]
    Input,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PendingCommand {
    SaveSession,
    ApproveCodeExec,
    DenyCodeExec,
    ExitCodeExec,
    StopCodeExec,
    ApplyFilePatch,
    CancelFilePatch,
    NewTab,
    NewCategory,
    OpenConversation,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CodeExecReasonTarget {
    Deny,
    Stop,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CodeExecSelectionTarget {
    Code,
    Stdout,
    Stderr,
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

#[derive(Default)]
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
    pub log_session_id: String,
    pub pending_code_exec: Option<PendingCodeExec>,
    pub code_exec_scroll: usize,
    pub code_exec_stdout_scroll: usize,
    pub code_exec_stderr_scroll: usize,
    pub code_exec_live: Option<Arc<Mutex<CodeExecLive>>>,
    pub code_exec_result_ready: bool,
    pub code_exec_finished_output: Option<String>,
    pub code_exec_cancel: Option<Arc<AtomicBool>>,
    pub code_exec_hover: Option<CodeExecHover>,
    pub code_exec_reason_target: Option<CodeExecReasonTarget>,
    pub code_exec_reason_input: TextArea<'static>,
    pub code_exec_container_id: Option<String>,
    pub code_exec_run_id: Option<String>,
    pub code_exec_selecting: Option<CodeExecSelectionTarget>,
    pub code_exec_code_selection: Option<crate::ui::selection::Selection>,
    pub code_exec_stdout_selection: Option<crate::ui::selection::Selection>,
    pub code_exec_stderr_selection: Option<crate::ui::selection::Selection>,
    pub pending_file_patch: Option<PendingFilePatch>,
    pub file_patch_scroll: usize,
    pub file_patch_hover: Option<FilePatchHover>,
    pub file_patch_selecting: bool,
    pub file_patch_selection: Option<crate::ui::selection::Selection>,
    pub pending_category_name: Option<String>,
    pub pending_open_conversation: Option<String>,
    pub terminal: Option<crate::ui::terminal::TerminalSession>,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub dirty_indices: Vec<usize>,
    pub cache_shift: Option<usize>,
    pub notice: Option<Notice>,
    pub command_suggestions: Vec<CommandSuggestion>,
    pub command_select: SelectionState,
}

#[derive(Clone, Debug)]
pub struct PendingCodeExec {
    pub call_id: String,
    pub language: String,
    pub code: String,
    pub exec_code: Option<String>,
    pub requested_at: Instant,
    pub stop_reason: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CodeExecLive {
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
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
    ReasonConfirm,
    ReasonBack,
}

#[derive(Clone, Debug)]
pub struct PendingFilePatch {
    pub call_id: String,
    pub path: Option<String>,
    pub diff: String,
    pub preview: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilePatchHover {
    Apply,
    Cancel,
}

impl App {
    pub fn new(system_prompt: &str, default_model: &str, default_prompt: &str) -> Self {
        let messages = build_initial_messages(system_prompt);
        base_app(messages, default_model, default_prompt)
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

    pub fn set_log_session_id(&mut self, id: &str) {
        self.log_session_id = id.to_string();
    }
}

fn build_initial_messages(system_prompt: &str) -> Vec<Message> {
    if system_prompt.trim().is_empty() {
        return Vec::new();
    }
    vec![Message {
        role: ROLE_SYSTEM.to_string(),
        content: system_prompt.to_string(),
        tool_call_id: None,
        tool_calls: None,
    }]
}

fn base_app(messages: Vec<Message>, default_model: &str, default_prompt: &str) -> App {
    App {
        messages,
        model_key: default_model.to_string(),
        prompt_key: default_prompt.to_string(),
        follow: true,
        next_request_id: 1,
        ..Default::default()
    }
}
