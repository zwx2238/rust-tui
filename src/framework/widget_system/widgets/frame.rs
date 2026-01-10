use crate::framework::widget_system::runtime::runtime_loop_steps::{
    FrameLayout, ProcessStreamUpdatesParams, active_frame_data, handle_pending_line, header_note,
    prepare_categories, process_stream_updates, tab_labels_and_pos,
};
use crate::framework::widget_system::runtime_tick::{ActiveFrameData, apply_preheat_results};
use std::error::Error;

use super::super::context::{UpdateCtx, UpdateOutput};

pub(crate) struct FrameLifecycle;

impl FrameLifecycle {
    pub(crate) fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        apply_preheat_results(&mut ctx.events.preheat, ctx.tabs);
        crate::framework::widget_system::widgets::terminal::apply_terminal_events(&mut ctx.events.terminal, ctx.tabs);
        let tabs = prepare_tabs(ctx);
        run_stream_updates(ctx, layout)?;
        let active_data = build_active_data(ctx, layout);
        let header_note = header_note(ctx.tabs, ctx.categories);
        handle_pending_actions(ctx, &active_data);
        Ok(UpdateOutput {
            active_data,
            tab_labels: tabs.labels,
            active_tab_pos: tabs.active_pos,
            header_note,
        })
    }
}

struct TabMeta {
    labels: Vec<String>,
    active_pos: usize,
}

fn prepare_tabs(ctx: &mut UpdateCtx<'_>) -> TabMeta {
    let active_category_name = prepare_categories(
        ctx.tabs,
        *ctx.active_tab,
        ctx.categories,
        ctx.active_category,
    );
    let (labels, active_pos) = tab_labels_and_pos(ctx.tabs, *ctx.active_tab, &active_category_name);
    TabMeta { labels, active_pos }
}

fn run_stream_updates(ctx: &mut UpdateCtx<'_>, layout: &FrameLayout) -> Result<(), Box<dyn Error>> {
    process_stream_updates(ProcessStreamUpdatesParams {
        llm_events: &mut ctx.events.llm,
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
    active_frame_data(crate::framework::widget_system::runtime::runtime_loop_steps::ActiveFrameDataParams {
        tabs: ctx.tabs,
        active_tab: *ctx.active_tab,
        args: ctx.args,
        theme: ctx.theme,
        msg_width: layout.layout.msg_width,
        view_height: layout.layout.view_height,
        input_area: layout.layout.input_area,
        startup_elapsed: *ctx.startup_elapsed,
    })
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
    crate::framework::widget_system::runtime_loop_helpers::handle_pending_command_if_any(
        crate::framework::widget_system::runtime_loop_helpers::HandlePendingCommandIfAnyParams {
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
        },
    );
}
