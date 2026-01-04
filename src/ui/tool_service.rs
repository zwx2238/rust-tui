use crate::args::Args;
use crate::model_registry::{ModelProfile, ModelRegistry};
use crate::types::{Message, ToolCall};
use crate::ui::net::UiEvent;
use crate::ui::runtime_code_exec::handle_code_exec_request;
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_requests::start_followup_request;
use crate::ui::tools::run_tool;
use std::sync::mpsc;

pub struct ToolService<'a> {
    registry: &'a ModelRegistry,
    args: &'a Args,
    tx: &'a mpsc::Sender<UiEvent>,
}

impl<'a> ToolService<'a> {
    pub fn new(registry: &'a ModelRegistry, args: &'a Args, tx: &'a mpsc::Sender<UiEvent>) -> Self {
        Self { registry, args, tx }
    }

    pub fn apply_tool_calls(&self, tab_state: &mut TabState, tab_id: usize, calls: &[ToolCall]) {
        let mut state = ToolApplyState::new(tab_state.app.tavily_api_key.clone());
        for call in calls {
            self.handle_tool_call(tab_state, tab_id, call, &mut state);
        }
        self.finalize_tool_calls(tab_state, tab_id, state);
    }
}

struct ToolApplyState {
    any_results: bool,
    needs_approval: bool,
    api_key: String,
}

impl ToolApplyState {
    fn new(api_key: String) -> Self {
        Self {
            any_results: false,
            needs_approval: false,
            api_key,
        }
    }
}

impl<'a> ToolService<'a> {
    fn handle_tool_call(
        &self,
        tab_state: &mut TabState,
        tab_id: usize,
        call: &ToolCall,
        state: &mut ToolApplyState,
    ) {
        match call.function.name.as_str() {
            "web_search" => self.handle_simple_tool(call, tab_state, state, ToolKind::WebSearch),
            "read_file" => self.handle_simple_tool(call, tab_state, state, ToolKind::ReadFile),
            "read_code" => self.handle_simple_tool(call, tab_state, state, ToolKind::ReadCode),
            "list_dir" => self.handle_simple_tool(call, tab_state, state, ToolKind::ListDir),
            "modify_file" => self.handle_modify_file(call, tab_state, tab_id, state),
            "code_exec" => self.handle_code_exec(call, tab_state, tab_id, state),
            _ => {}
        }
    }

    fn handle_simple_tool(
        &self,
        call: &ToolCall,
        tab_state: &mut TabState,
        state: &mut ToolApplyState,
        kind: ToolKind,
    ) {
        if !self.tool_enabled(kind) {
            let msg = format!(r#"{{"error":"{} 未启用"}}"#, call.function.name);
            push_tool_message(tab_state, call, msg);
            state.any_results = true;
            return;
        }
        let root = tool_root(self.args.yolo_enabled());
        let result = run_tool(call, &state.api_key, root.as_deref());
        push_tool_message(tab_state, call, result.content);
        if result.has_results {
            state.any_results = true;
        }
    }

    fn handle_modify_file(
        &self,
        call: &ToolCall,
        tab_state: &mut TabState,
        tab_id: usize,
        state: &mut ToolApplyState,
    ) {
        if self.reject_modify_file(tab_state, call, state) {
            return;
        }
        match crate::ui::runtime_file_patch::handle_file_patch_request(tab_state, call) {
            Ok(()) => self.apply_file_patch(tab_state, tab_id, state),
            Err(err) => push_tool_error(tab_state, call, state, err),
        }
    }

    fn handle_code_exec(
        &self,
        call: &ToolCall,
        tab_state: &mut TabState,
        tab_id: usize,
        state: &mut ToolApplyState,
    ) {
        if !self.args.code_exec_enabled() {
            push_tool_error(tab_state, call, state, "code_exec 未启用");
            return;
        }
        match handle_code_exec_request(tab_state, call) {
            Ok(()) => self.apply_code_exec(tab_state, tab_id, state),
            Err(err) => push_tool_error(tab_state, call, state, err),
        }
    }

    fn tool_enabled(&self, kind: ToolKind) -> bool {
        match kind {
            ToolKind::WebSearch => self.args.web_search_enabled(),
            ToolKind::ReadFile => self.args.read_file_enabled(),
            ToolKind::ReadCode => self.args.read_code_enabled(),
            ToolKind::ListDir => self.args.read_file_enabled(),
        }
    }

    fn finalize_tool_calls(&self, tab_state: &mut TabState, tab_id: usize, state: ToolApplyState) {
        if state.needs_approval {
            return;
        }
        if !state.any_results {
            push_assistant_message(tab_state, "未找到可靠结果，无法确认。".to_string());
            return;
        }
        let model = self.resolve_model(tab_state);
        self.start_followup(tab_state, tab_id, model);
    }

    fn reject_modify_file(
        &self,
        tab_state: &mut TabState,
        call: &ToolCall,
        state: &mut ToolApplyState,
    ) -> bool {
        if self.args.read_only_enabled() {
            push_tool_error(tab_state, call, state, "read_only 模式禁止 modify_file");
            return true;
        }
        if !self.args.modify_file_enabled() {
            push_tool_error(tab_state, call, state, "modify_file 未启用");
            return true;
        }
        false
    }

    fn apply_file_patch(
        &self,
        tab_state: &mut TabState,
        tab_id: usize,
        state: &mut ToolApplyState,
    ) {
        if self.args.yolo_enabled() {
            crate::ui::runtime_file_patch::handle_file_patch_apply(
                tab_state,
                tab_id,
                self.registry,
                self.args,
                self.tx,
            );
        } else {
            state.needs_approval = true;
        }
        state.any_results = true;
    }

    fn apply_code_exec(&self, tab_state: &mut TabState, tab_id: usize, state: &mut ToolApplyState) {
        if self.args.yolo_enabled() {
            crate::ui::runtime_code_exec::handle_code_exec_approve(
                tab_state,
                tab_id,
                self.registry,
                self.args,
                self.tx,
            );
        } else {
            state.needs_approval = true;
        }
        state.any_results = true;
    }

    fn resolve_model(&self, tab_state: &TabState) -> &ModelProfile {
        self.registry
            .get(&tab_state.app.model_key)
            .unwrap_or_else(|| {
                self.registry
                    .get(&self.registry.default_key)
                    .expect("model")
            })
    }

    fn start_followup(&self, tab_state: &mut TabState, tab_id: usize, model: &ModelProfile) {
        let log_session_id = tab_state.app.log_session_id.clone();
        start_followup_request(crate::ui::runtime_requests::StartFollowupRequestParams {
            tab_state,
            base_url: &model.base_url,
            api_key: &model.api_key,
            model: &model.model,
            _show_reasoning: self.args.show_reasoning,
            tx: self.tx,
            tab_id,
            enable_web_search: self.args.web_search_enabled(),
            enable_code_exec: self.args.code_exec_enabled(),
            enable_read_file: self.args.read_file_enabled(),
            enable_read_code: self.args.read_code_enabled(),
            enable_modify_file: self.args.modify_file_enabled(),
            log_requests: self.args.log_requests.clone(),
            log_session_id,
        });
    }
}

enum ToolKind {
    WebSearch,
    ReadFile,
    ReadCode,
    ListDir,
}

fn tool_root(yolo: bool) -> Option<std::path::PathBuf> {
    if yolo {
        std::env::current_dir().ok()
    } else {
        None
    }
}

fn push_tool_message(tab_state: &mut TabState, call: &ToolCall, content: String) {
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_TOOL.to_string(),
        content,
        tool_call_id: Some(call.id.clone()),
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
}

fn push_tool_error(
    tab_state: &mut TabState,
    call: &ToolCall,
    state: &mut ToolApplyState,
    message: impl AsRef<str>,
) {
    push_tool_message(
        tab_state,
        call,
        format!(r#"{{"error":"{}"}}"#, message.as_ref()),
    );
    state.any_results = true;
}

fn push_assistant_message(tab_state: &mut TabState, content: String) {
    let idx = tab_state.app.messages.len();
    tab_state.app.messages.push(Message {
        role: crate::types::ROLE_ASSISTANT.to_string(),
        content,
        tool_call_id: None,
        tool_calls: None,
    });
    tab_state.app.dirty_indices.push(idx);
}
