use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use crate::ui::runtime_loop_iteration::{
    LoopControl, RunLoopIterationParams, run_loop_iteration,
};
use crate::ui::runtime_view::ViewState;
use crate::ui::widget_system::WidgetSystem;
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
    pub rx: &'a mpsc::Receiver<UiEvent>,
    pub tx: &'a mpsc::Sender<UiEvent>,
    pub preheat_tx: &'a mpsc::Sender<PreheatTask>,
    pub preheat_res_rx: &'a mpsc::Receiver<PreheatResult>,
    pub registry: &'a crate::model_registry::ModelRegistry,
    pub prompt_registry: &'a crate::llm::prompts::PromptRegistry,
    pub args: &'a Args,
    pub theme: &'a RenderTheme,
    pub start_time: Instant,
}

pub(crate) fn run_loop(mut params: RunLoopParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let mut startup_elapsed = None;
    let mut view = ViewState::new();
    let mut widget_system = WidgetSystem::new();
    loop {
        if run_loop_once(&mut params, &mut view, &mut startup_elapsed, &mut widget_system)? {
            break;
        }
    }
    Ok(())
}

fn run_loop_once(
    params: &mut RunLoopParams<'_>,
    view: &mut ViewState,
    startup_elapsed: &mut Option<std::time::Duration>,
    widget_system: &mut WidgetSystem,
) -> Result<bool, Box<dyn std::error::Error>> {
    let control = run_loop_iteration(RunLoopIterationParams {
        widget_system,
        terminal: params.terminal,
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        session_location: params.session_location,
        rx: params.rx,
        tx: params.tx,
        preheat_tx: params.preheat_tx,
        preheat_res_rx: params.preheat_res_rx,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        theme: params.theme,
        start_time: params.start_time,
        startup_elapsed,
        view,
    })?;
    Ok(control == LoopControl::Break || should_stop_after_iteration())
}

fn should_stop_after_iteration() -> bool {
    #[cfg(test)]
    if std::env::var("DEEPCHAT_TEST_RUN_LOOP_ONCE").is_ok() {
        return true;
    }
    false
}
