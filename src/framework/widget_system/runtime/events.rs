use crate::types::{ToolCall, Usage};
use crate::framework::widget_system::runtime::runtime_helpers::PreheatResult;
use crossterm::event::Event as CrosstermEvent;
use std::sync::mpsc::Sender;

pub(crate) enum LlmEvent {
    Chunk(String),
    ReasoningChunk(String),
    Error(String),
    Done {
        usage: Option<Usage>,
    },
    ToolCalls {
        calls: Vec<ToolCall>,
        usage: Option<Usage>,
    },
}

pub(crate) struct UiEvent {
    pub(crate) tab: String,
    pub(crate) request_id: u64,
    pub(crate) event: LlmEvent,
}

pub(crate) enum RuntimeEvent {
    Input(CrosstermEvent),
    Llm(UiEvent),
    Preheat(PreheatResult),
    Terminal(crate::framework::widget_system::widgets::terminal::TerminalEvent),
}

pub(crate) struct EventBatch {
    pub(crate) input: Vec<CrosstermEvent>,
    pub(crate) llm: Vec<UiEvent>,
    pub(crate) preheat: Vec<PreheatResult>,
    pub(crate) terminal: Vec<crate::framework::widget_system::widgets::terminal::TerminalEvent>,
}

impl EventBatch {
    pub(crate) fn new() -> Self {
        Self {
            input: Vec::new(),
            llm: Vec::new(),
            preheat: Vec::new(),
            terminal: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, event: RuntimeEvent) {
        match event {
            RuntimeEvent::Input(e) => self.input.push(e),
            RuntimeEvent::Llm(e) => self.llm.push(e),
            RuntimeEvent::Preheat(e) => self.preheat.push(e),
            RuntimeEvent::Terminal(e) => self.terminal.push(e),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.input.clear();
        self.llm.clear();
        self.preheat.clear();
        self.terminal.clear();
    }
}

pub(crate) fn send_llm(tx: &Sender<RuntimeEvent>, tab: String, request_id: u64, event: LlmEvent) {
    let _ = tx.send(RuntimeEvent::Llm(UiEvent {
        tab,
        request_id,
        event,
    }));
}

pub(crate) fn send_preheat(tx: &Sender<RuntimeEvent>, result: PreheatResult) {
    let _ = tx.send(RuntimeEvent::Preheat(result));
}
