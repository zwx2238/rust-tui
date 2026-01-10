use super::{RenderSnapshot, RunLoopParams};
use crate::ui::events::EventBatch;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::runtime_view::ViewState;
use crate::framework::widget_system::{LayoutCtx, RenderCtx, UpdateCtx, UpdateOutput, WidgetSystem};

pub(super) fn build_snapshot(
    params: &mut RunLoopParams<'_>,
    view: &mut ViewState,
    widget_system: &mut WidgetSystem,
    startup_elapsed: &mut Option<std::time::Duration>,
    events: &mut EventBatch,
) -> Result<RenderSnapshot, Box<dyn std::error::Error>> {
    let layout = build_layout(params, view, widget_system)?;
    let update = run_update(
        params,
        view,
        widget_system,
        startup_elapsed,
        events,
        &layout,
    )?;
    let jump_rows = run_render(
        params,
        view,
        widget_system,
        startup_elapsed,
        &layout,
        &update,
    )?;
    Ok(RenderSnapshot {
        layout,
        update,
        jump_rows,
    })
}

fn build_layout(
    params: &mut RunLoopParams<'_>,
    view: &ViewState,
    widget_system: &mut WidgetSystem,
) -> Result<FrameLayout, Box<dyn std::error::Error>> {
    let mut ctx = LayoutCtx {
        terminal: params.terminal,
        view,
        tabs: params.tabs,
        active_tab: *params.active_tab,
        categories: params.categories,
    };
    widget_system.layout(&mut ctx)
}

fn run_update(
    params: &mut RunLoopParams<'_>,
    view: &mut ViewState,
    widget_system: &mut WidgetSystem,
    startup_elapsed: &mut Option<std::time::Duration>,
    events: &mut EventBatch,
    layout: &FrameLayout,
) -> Result<UpdateOutput, Box<dyn std::error::Error>> {
    let mut ctx = UpdateCtx {
        tabs: params.tabs,
        active_tab: params.active_tab,
        categories: params.categories,
        active_category: params.active_category,
        session_location: params.session_location,
        tx: params.tx,
        preheat_tx: params.preheat_tx,
        events,
        registry: params.registry,
        prompt_registry: params.prompt_registry,
        args: params.args,
        theme: params.theme,
        startup_elapsed,
        view,
    };
    widget_system.update(&mut ctx, layout)
}

fn run_render(
    params: &mut RunLoopParams<'_>,
    view: &mut ViewState,
    widget_system: &mut WidgetSystem,
    startup_elapsed: &mut Option<std::time::Duration>,
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
        view,
        start_time: params.start_time,
        startup_elapsed,
    };
    widget_system.render(&mut ctx, layout, update)
}
