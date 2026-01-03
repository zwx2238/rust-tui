use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use crate::ui::runtime_tick::ActiveFrameData;
use crate::ui::runtime_view::ViewState;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::Instant;

pub(crate) struct WidgetFrame<'a, 'b> {
    pub(crate) ctx: &'a mut crate::ui::render_context::RenderContext<'b>,
    pub(crate) view: &'a mut ViewState,
    pub(crate) jump_rows: &'a mut Vec<crate::ui::jump::JumpRow>,
}

pub(crate) struct LayoutCtx<'a> {
    pub(crate) terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    pub(crate) view: &'a ViewState,
    pub(crate) tabs: &'a [TabState],
    pub(crate) active_tab: usize,
    pub(crate) categories: &'a [String],
}

pub(crate) struct UpdateCtx<'a> {
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) categories: &'a mut Vec<String>,
    pub(crate) active_category: &'a mut usize,
    pub(crate) session_location: &'a mut Option<crate::session::SessionLocation>,
    pub(crate) rx: &'a mpsc::Receiver<UiEvent>,
    pub(crate) tx: &'a mpsc::Sender<UiEvent>,
    pub(crate) preheat_tx: &'a mpsc::Sender<PreheatTask>,
    pub(crate) preheat_res_rx: &'a mpsc::Receiver<PreheatResult>,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) args: &'a Args,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) startup_elapsed: &'a mut Option<std::time::Duration>,
    pub(crate) view: &'a mut ViewState,
}

pub(crate) struct RenderCtx<'a> {
    pub(crate) terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: usize,
    pub(crate) categories: &'a [String],
    pub(crate) active_category: usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) view: &'a mut ViewState,
    pub(crate) start_time: Instant,
    pub(crate) startup_elapsed: &'a mut Option<std::time::Duration>,
}

pub(crate) struct EventCtx<'a> {
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: &'a mut usize,
    pub(crate) categories: &'a mut Vec<String>,
    pub(crate) active_category: &'a mut usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) registry: &'a crate::model_registry::ModelRegistry,
    pub(crate) prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub(crate) args: &'a Args,
    pub(crate) view: &'a mut ViewState,
}

pub(crate) struct UpdateOutput {
    pub(crate) active_data: ActiveFrameData,
    pub(crate) tab_labels: Vec<String>,
    pub(crate) active_tab_pos: usize,
    pub(crate) header_note: Option<String>,
}
