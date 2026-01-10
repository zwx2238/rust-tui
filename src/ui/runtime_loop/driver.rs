use super::{RenderSnapshot, RunLoopParams, should_stop_after_iteration};
use crate::ui::events::EventBatch;
use crate::ui::runtime_helpers::PreheatResult;
use crate::ui::runtime_loop::{event_wait, snapshot};
use crate::ui::runtime_view::ViewState;
use crate::framework::widget_system::WidgetSystem;

pub(super) fn run_loop(params: &mut RunLoopParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = LoopState::init(params)?;
    if should_stop_after_iteration() {
        return Ok(());
    }
    loop {
        if state.step(params)? {
            return Ok(());
        }
        if should_stop_after_iteration() {
            return Ok(());
        }
    }
}

struct LoopState {
    startup_elapsed: Option<std::time::Duration>,
    view: ViewState,
    widget_system: WidgetSystem,
    events: EventBatch,
    snapshot: RenderSnapshot,
}

impl LoopState {
    fn init(params: &mut RunLoopParams<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut startup_elapsed = None;
        let mut view = ViewState::new();
        let mut widget_system = WidgetSystem::new();
        let mut events = EventBatch::new();
        let snapshot = snapshot::build_snapshot(
            params,
            &mut view,
            &mut widget_system,
            &mut startup_elapsed,
            &mut events,
        )?;
        Ok(Self {
            startup_elapsed,
            view,
            widget_system,
            events,
            snapshot,
        })
    }

    fn step(&mut self, params: &mut RunLoopParams<'_>) -> Result<bool, Box<dyn std::error::Error>> {
        let outcome = collect_events(params, self);
        if outcome.disconnected {
            return Ok(true);
        }
        if dispatch_inputs(params, self)? {
            return Ok(true);
        }
        apply_preheat_results(&mut self.events.preheat, params.tabs);
        if !should_render(&outcome, params, self) {
            self.events.clear();
            return Ok(false);
        }
        self.events.input.clear();
        self.snapshot = snapshot::build_snapshot(
            params,
            &mut self.view,
            &mut self.widget_system,
            &mut self.startup_elapsed,
            &mut self.events,
        )?;
        Ok(false)
    }
}

fn collect_events(params: &RunLoopParams<'_>, state: &mut LoopState) -> event_wait::WaitOutcome {
    let timeout = event_wait::compute_timeout(params.tabs, *params.active_tab);
    event_wait::wait_for_events(params.rx, timeout, &mut state.events)
}

fn dispatch_inputs(
    params: &mut RunLoopParams<'_>,
    state: &mut LoopState,
) -> Result<bool, Box<dyn std::error::Error>> {
    dispatch_input_events(params, state)
}

fn should_render(
    outcome: &event_wait::WaitOutcome,
    params: &RunLoopParams<'_>,
    state: &LoopState,
) -> bool {
    if outcome.ticked {
        return true;
    }
    if event_wait::input_batch_dirty(&state.events.input) {
        return true;
    }
    if event_wait::preheat_touches_active_tab(&state.events.preheat, *params.active_tab) {
        return true;
    }
    !state.events.llm.is_empty() || !state.events.terminal.is_empty()
}

fn dispatch_input_events(
    params: &mut RunLoopParams<'_>,
    state: &mut LoopState,
) -> Result<bool, Box<dyn std::error::Error>> {
    if state.events.input.is_empty() {
        return Ok(false);
    }
    let widget_system = &mut state.widget_system;
    let snapshot = &state.snapshot;
    let mut ctx = crate::framework::widget_system::EventCtx {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        theme: params.theme,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        view: &mut state.view,
    };
    for ev in &state.events.input {
        if dispatch_one_input(widget_system, &mut ctx, snapshot, ev)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn dispatch_one_input(
    widget_system: &mut WidgetSystem,
    ctx: &mut crate::framework::widget_system::EventCtx<'_>,
    snapshot: &RenderSnapshot,
    ev: &crossterm::event::Event,
) -> Result<bool, Box<dyn std::error::Error>> {
    widget_system.event(
        ctx,
        &snapshot.layout,
        &snapshot.update,
        &snapshot.jump_rows,
        ev,
    )
}

fn apply_preheat_results(
    preheat: &mut Vec<PreheatResult>,
    tabs: &mut [crate::ui::runtime_helpers::TabState],
) {
    crate::ui::runtime_tick::apply_preheat_results(preheat, tabs);
}
