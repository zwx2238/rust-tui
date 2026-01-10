mod driver;
mod event_wait;
mod snapshot;

use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::events::RuntimeEvent;
use crate::ui::runtime_helpers::{PreheatTask, TabState};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::UpdateOutput;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::Instant;

pub(crate) struct RunLoopParams<'a> {
    pub terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    pub tabs: &'a mut Vec<TabState>,
    pub active_tab: &'a mut usize,
    pub categories: &'a mut Vec<String>,
    pub active_category: &'a mut usize,
    pub session_location: &'a mut Option<crate::session::SessionLocation>,
    pub rx: &'a mpsc::Receiver<RuntimeEvent>,
    pub tx: &'a mpsc::Sender<RuntimeEvent>,
    pub preheat_tx: &'a mpsc::Sender<PreheatTask>,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
    pub theme: &'a RenderTheme,
    pub start_time: Instant,
}

pub(crate) struct RenderSnapshot {
    pub(crate) layout: FrameLayout,
    pub(crate) update: UpdateOutput,
    pub(crate) jump_rows: Vec<crate::ui::jump::JumpRow>,
}

pub(crate) fn run_loop(mut params: RunLoopParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    driver::run_loop(&mut params)
}

pub(super) fn should_stop_after_iteration() -> bool {
    false
}
