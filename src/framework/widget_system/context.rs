use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::events::{EventBatch, RuntimeEvent};
use crate::ui::runtime_helpers::{PreheatTask, TabState};
use crate::ui::runtime_tick::ActiveFrameData;
use crate::ui::runtime_view::ViewState;
use crate::ui::state::App;
use ratatui::layout::Rect;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::Instant;

pub(crate) struct RenderState<'a> {
    pub(crate) tabs: &'a mut Vec<TabState>,
    pub(crate) active_tab: usize,
    pub(crate) tab_labels: &'a [String],
    pub(crate) active_tab_pos: usize,
    pub(crate) categories: &'a [String],
    pub(crate) active_category: usize,
    pub(crate) theme: &'a RenderTheme,
    pub(crate) startup_text: Option<&'a str>,
    pub(crate) full_area: Rect,
    pub(crate) msg_area: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) category_area: Rect,
    pub(crate) header_area: Rect,
    pub(crate) footer_area: Rect,
    pub(crate) input_area: Rect,
    pub(crate) msg_width: usize,
    pub(crate) text: &'a ratatui::text::Text<'a>,
    pub(crate) total_lines: usize,
    pub(crate) header_note: Option<&'a str>,
    pub(crate) models: &'a [crate::model_registry::ModelProfile],
    pub(crate) prompts: &'a [crate::llm::prompts::SystemPrompt],
}

impl RenderState<'_> {
    pub(crate) fn with_active_tab<T>(&self, f: impl FnOnce(&TabState) -> T) -> Option<T> {
        self.tabs.get(self.active_tab).map(f)
    }

    pub(crate) fn active_app(&self) -> Option<&App> {
        self.tabs.get(self.active_tab).map(|tab| &tab.app)
    }

    pub(crate) fn active_app_mut(&mut self) -> Option<&mut App> {
        self.tabs.get_mut(self.active_tab).map(|tab| &mut tab.app)
    }

    pub(crate) fn tabs(&self) -> &[TabState] {
        self.tabs
    }
}

pub(crate) struct WidgetFrame<'frame, 'state, 'data, 'buf> {
    pub(crate) frame: &'frame mut ratatui::Frame<'buf>,
    pub(crate) state: &'state mut RenderState<'data>,
    pub(crate) view: &'state mut ViewState,
    pub(crate) jump_rows: &'state mut Vec<crate::ui::jump::JumpRow>,
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
    pub(crate) tx: &'a mpsc::Sender<RuntimeEvent>,
    pub(crate) preheat_tx: &'a mpsc::Sender<PreheatTask>,
    pub(crate) events: &'a mut EventBatch,
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
