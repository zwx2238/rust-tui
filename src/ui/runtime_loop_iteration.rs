use crate::args::Args;
use crate::render::RenderTheme;
use crate::ui::net::UiEvent;
use crate::ui::runtime_helpers::{PreheatResult, PreheatTask, TabState};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::runtime_view::ViewState;
use crate::ui::widget_system::{
    EventCtx, LayoutCtx, RenderCtx, UpdateCtx, UpdateOutput, WidgetSystem,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::mpsc;
use std::time::Instant;

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum LoopControl {
    Continue,
    Break,
}

pub(crate) struct RunLoopIterationParams<'a> {
    pub(crate) widget_system: &'a mut WidgetSystem,
    pub(crate) terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
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
    pub(crate) start_time: Instant,
    pub(crate) startup_elapsed: &'a mut Option<std::time::Duration>,
    pub(crate) view: &'a mut ViewState,
}

pub(crate) fn run_loop_iteration(
    mut params: RunLoopIterationParams<'_>,
) -> Result<LoopControl, Box<dyn std::error::Error>> {
    let layout = build_layout(&mut params)?;
    let update = run_update(&mut params, &layout)?;
    let jump_rows = run_render(&mut params, &layout, &update)?;
    if run_event(&mut params, &layout, &update, &jump_rows)? {
        return Ok(LoopControl::Break);
    }
    Ok(LoopControl::Continue)
}

fn build_layout(
    params: &mut RunLoopIterationParams<'_>,
) -> Result<FrameLayout, Box<dyn std::error::Error>> {
    let mut ctx = LayoutCtx {
        terminal: params.terminal,
        view: params.view,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        categories: params.categories,
    };
    params.widget_system.layout(&mut ctx)
}

fn run_update(
    params: &mut RunLoopIterationParams<'_>,
    layout: &FrameLayout,
) -> Result<UpdateOutput, Box<dyn std::error::Error>> {
    let mut ctx = UpdateCtx {
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
        startup_elapsed: params.startup_elapsed,
        view: params.view,
    };
    params.widget_system.update(&mut ctx, layout)
}

fn run_render(
    params: &mut RunLoopIterationParams<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
) -> Result<Vec<crate::ui::jump::JumpRow>, Box<dyn std::error::Error>> {
    let mut ctx = RenderCtx {
        terminal: params.terminal,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        categories: params.categories,
        active_category: *params.active_category,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        view: params.view,
        start_time: params.start_time,
        startup_elapsed: params.startup_elapsed,
    };
    params.widget_system.render(&mut ctx, layout, update)
}

fn run_event(
    params: &mut RunLoopIterationParams<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut ctx = EventCtx {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        view: params.view,
    };
    params
        .widget_system
        .event(&mut ctx, layout, update, jump_rows)
}
