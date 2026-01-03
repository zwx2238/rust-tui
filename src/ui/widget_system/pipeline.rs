use crate::ui::runtime_loop_helpers::{
    HandlePendingCommandIfAnyParams, handle_pending_command_if_any,
};
use crate::ui::runtime_loop_steps::{
    DispatchContextParams, FrameLayout, LayoutContextParams, ProcessStreamUpdatesParams,
    active_frame_data, frame_layout, handle_pending_line, header_note, poll_and_dispatch_event,
    prepare_categories, process_stream_updates, tab_labels_and_pos,
};
use crate::ui::runtime_tick::{ActiveFrameData, drain_preheat_results};
use std::error::Error;

use super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput};

pub(crate) fn layout_root(ctx: &mut LayoutCtx<'_>) -> Result<FrameLayout, Box<dyn Error>> {
    frame_layout(ctx.terminal, ctx.view, ctx.tabs, ctx.active_tab, ctx.categories)
}

pub(crate) fn update_root(
    ctx: &mut UpdateCtx<'_>,
    layout: &FrameLayout,
) -> Result<UpdateOutput, Box<dyn Error>> {
    drain_preheat_results(ctx.preheat_res_rx, ctx.tabs);
    let (tab_labels, active_tab_pos) = prepare_tabs(ctx);
    run_stream_updates(ctx, layout)?;
    let active_data = build_active_data(ctx, layout);
    let header_note = header_note(ctx.tabs, ctx.categories);
    handle_pending_actions(ctx, &active_data);
    Ok(UpdateOutput {
        active_data,
        tab_labels,
        active_tab_pos,
        header_note,
    })
}

pub(crate) fn event_root(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    jump_rows: &[crate::ui::jump::JumpRow],
) -> Result<bool, Box<dyn Error>> {
    let mut dispatch = prepare_event_dispatch(ctx, layout);
    let layout_params = build_layout_params(layout, update);
    poll_and_dispatch_event(&mut dispatch.params, layout_params, dispatch.view, jump_rows)
}

fn prepare_tabs(ctx: &mut UpdateCtx<'_>) -> (Vec<String>, usize) {
    let active_category_name = prepare_categories(
        ctx.tabs,
        *ctx.active_tab,
        ctx.categories,
        ctx.active_category,
    );
    tab_labels_and_pos(ctx.tabs, *ctx.active_tab, &active_category_name)
}

fn run_stream_updates(ctx: &mut UpdateCtx<'_>, layout: &FrameLayout) -> Result<(), Box<dyn Error>> {
    process_stream_updates(ProcessStreamUpdatesParams {
        rx: ctx.rx,
        tabs: ctx.tabs,
        active_tab: *ctx.active_tab,
        theme: ctx.theme,
        msg_width: layout.layout.msg_width,
        registry: ctx.registry,
        args: ctx.args,
        tx: ctx.tx,
        preheat_tx: ctx.preheat_tx,
        view: ctx.view,
    })
}

fn build_active_data(ctx: &mut UpdateCtx<'_>, layout: &FrameLayout) -> ActiveFrameData {
    active_frame_data(
        ctx.tabs,
        *ctx.active_tab,
        ctx.theme,
        layout.layout.msg_width,
        layout.layout.view_height,
        layout.layout.input_area,
        *ctx.startup_elapsed,
    )
}

fn handle_pending_actions(ctx: &mut UpdateCtx<'_>, active_data: &ActiveFrameData) {
    handle_pending_line(
        active_data.pending_line.clone(),
        ctx.tabs,
        *ctx.active_tab,
        ctx.registry,
        ctx.args,
        ctx.tx,
    );
    handle_pending_command_if_any(HandlePendingCommandIfAnyParams {
        pending_command: active_data.pending_command,
        tabs: ctx.tabs,
        active_tab: ctx.active_tab,
        categories: ctx.categories,
        active_category: ctx.active_category,
        session_location: ctx.session_location,
        registry: ctx.registry,
        prompt_registry: ctx.prompt_registry,
        args: ctx.args,
        tx: ctx.tx,
    });
}

fn build_layout_params(layout: &FrameLayout, update: &UpdateOutput) -> LayoutContextParams {
    LayoutContextParams {
        size: layout.size,
        tabs_area: layout.layout.tabs_area,
        msg_area: layout.layout.msg_area,
        input_area: layout.layout.input_area,
        category_area: layout.layout.category_area,
        view_height: layout.layout.view_height,
        total_lines: update.active_data.total_lines,
    }
}

struct EventDispatch<'a> {
    params: DispatchContextParams<'a>,
    view: &'a mut crate::ui::runtime_view::ViewState,
}

fn prepare_event_dispatch<'a>(
    ctx: &'a mut EventCtx<'_>,
    layout: &FrameLayout,
) -> EventDispatch<'a> {
    let EventCtx {
        tabs,
        active_tab,
        categories,
        active_category,
        theme,
        registry,
        prompt_registry,
        args,
        view,
    } = ctx;
    let params = DispatchContextParams {
        tabs,
        active_tab,
        categories,
        active_category,
        msg_width: layout.layout.msg_width,
        theme,
        registry,
        prompt_registry,
        args,
    };
    EventDispatch { params, view }
}
