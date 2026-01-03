use crate::ui::jump::JumpRow;
use crate::ui::runtime_dispatch::{handle_key_event_loop, handle_mouse_event_loop};
use crate::ui::command_suggestions::refresh_command_suggestions;
use crate::ui::state::Focus;
use crate::ui::runtime_loop_helpers::{HandlePendingCommandIfAnyParams, handle_pending_command_if_any};
use crate::ui::runtime_loop_steps::{
    FrameLayout, ProcessStreamUpdatesParams, active_frame_data, frame_layout, handle_pending_line,
    header_note, prepare_categories, process_stream_updates, tab_labels_and_pos,
};
use crate::ui::runtime_tick::{ActiveFrameData, drain_preheat_results};
use std::error::Error;

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::bindings::{bind_active_tab, bind_event};
use super::super::events::poll_event;
use super::super::lifecycle::Widget;

pub(crate) struct ChatWidget;

impl Widget for ChatWidget {
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>) -> Result<FrameLayout, Box<dyn Error>> {
        frame_layout(ctx.terminal, ctx.view, ctx.tabs, ctx.active_tab, ctx.categories)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        drain_preheat_results(ctx.preheat_res_rx, ctx.tabs);
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

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
    ) -> Result<bool, Box<dyn Error>> {
        let Some(event) = poll_event()? else {
            return Ok(false);
        };
        let mut binding = bind_event(ctx, layout, update);
        match event {
            crossterm::event::Event::Key(key) => {
                handle_key_event_loop(key, &mut binding.dispatch, binding.layout, binding.view, jump_rows)
            }
            crossterm::event::Event::Paste(paste) => {
                if binding.view.is_chat() {
                    if let Some(mut active) = bind_active_tab(binding.dispatch.tabs, *binding.dispatch.active_tab) {
                        handle_paste_with_binding(&mut active, &paste);
                    }
                }
                Ok(false)
            }
            crossterm::event::Event::Mouse(m) => {
                handle_mouse_event_loop(m, &mut binding.dispatch, binding.layout, binding.view, jump_rows);
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        crate::ui::overlay_render_base::render_chat_view(frame.ctx)
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

fn handle_paste_with_binding(
    binding: &mut super::super::bindings::ActiveTabBinding<'_>,
    paste: &str,
) {
    let app = binding.app();
    if app.focus == Focus::Input && !app.busy {
        let text = paste.replace("\r\n", "\n").replace('\r', "\n");
        app.input.insert_str(text);
        refresh_command_suggestions(app);
    }
}
