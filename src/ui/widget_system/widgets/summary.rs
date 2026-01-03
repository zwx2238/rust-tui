use crate::ui::jump::JumpRow;
use crate::ui::overlay_table_state::{OverlayAreas, OverlayRowCounts, with_active_table_handle};
use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext, apply_model_selection, apply_prompt_selection};
use crate::ui::runtime_dispatch::key_helpers::{handle_pre_key_actions, handle_view_action_flow, is_quit_key, resolve_view_action};
use crate::ui::runtime_events::handle_tab_category_click;
use crate::ui::runtime_view::{ViewAction, apply_view_action, handle_view_mouse};
use crate::ui::scroll::SCROLL_STEP_I32;
use crate::ui::shortcut_help::help_rows_len;
use crate::ui::runtime_loop_steps::{FrameLayout, frame_layout};
use std::error::Error;

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::bindings::bind_event;
use super::super::events::poll_event;
use super::super::lifecycle::Widget;

pub(crate) struct SummaryWidget;

impl Widget for SummaryWidget {
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>) -> Result<FrameLayout, Box<dyn Error>> {
        frame_layout(ctx.terminal, ctx.view, ctx.tabs, ctx.active_tab, ctx.categories)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        super::overlay_update::update_overlay(ctx, layout, update_overlay_state)
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
    ) -> Result<bool, Box<dyn Error>> {
        overlay_event(ctx, layout, update, jump_rows)
    }

    fn render(&mut self, frame: &mut WidgetFrame<'_, '_>) -> Result<(), Box<dyn Error>> {
        clamp_overlay_tables(frame.view, frame.ctx, frame.jump_rows.len());
        crate::ui::overlay_render::render_summary_overlay(frame.ctx, frame.view)?;
        Ok(())
    }
}

fn update_overlay_state(_ctx: &mut UpdateCtx<'_>) {}

fn clamp_overlay_tables(
    view: &mut crate::ui::runtime_view::ViewState,
    ctx: &crate::ui::render_context::RenderContext<'_>,
    jump_len: usize,
) {
    let areas = OverlayAreas {
        full: ctx.full_area,
        msg: ctx.msg_area,
    };
    let counts = OverlayRowCounts {
        tabs: ctx.tabs.len(),
        jump: jump_len,
        models: ctx.models.len(),
        prompts: ctx.prompts.len(),
        help: help_rows_len(),
    };
    let _ = with_active_table_handle(view, areas, counts, |mut handle| handle.clamp());
}

fn overlay_event(
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
            if is_quit_key(key) {
                return Ok(true);
            }
            if handle_pre_key_actions(&mut binding.dispatch, binding.view, key) {
                return Ok(false);
            }
            let action = resolve_view_action(&mut binding.dispatch, binding.view, key, jump_rows);
            if handle_view_action_flow(&mut binding.dispatch, binding.layout, binding.view, jump_rows, action, key) {
                return Ok(false);
            }
            Ok(false)
        }
        crossterm::event::Event::Mouse(m) => {
            handle_overlay_mouse(m, &mut binding.dispatch, binding.layout, binding.view, jump_rows);
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn handle_overlay_mouse(
    m: crossterm::event::MouseEvent,
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    view: &mut crate::ui::runtime_view::ViewState,
    jump_rows: &[JumpRow],
) {
    if handle_tab_click(ctx, layout, m) {
        return;
    }
    if handle_overlay_scroll(view, ctx, layout, jump_rows.len(), m.kind) {
        return;
    }
    let row = overlay_row_at(view, ctx, layout, jump_rows.len(), m.column, m.row);
    let action = handle_view_mouse(view, row, ctx.tabs.len(), jump_rows.len(), m.kind);
    if let ViewAction::SelectModel(idx) = action {
        apply_model_selection(ctx, idx);
        return;
    }
    if let ViewAction::SelectPrompt(idx) = action {
        apply_prompt_selection(ctx, idx);
        return;
    }
    let _ = apply_view_action(
        action,
        jump_rows,
        ctx.tabs,
        ctx.active_tab,
        ctx.categories,
        ctx.active_category,
    );
}

fn handle_tab_click(
    ctx: &mut DispatchContext<'_>,
    layout: LayoutContext,
    m: crossterm::event::MouseEvent,
) -> bool {
    if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
        return false;
    }
    handle_tab_category_click(crate::ui::runtime_events::TabCategoryClickParams {
        mouse_x: m.column,
        mouse_y: m.row,
        tabs: ctx.tabs,
        active_tab: ctx.active_tab,
        categories: ctx.categories,
        active_category: ctx.active_category,
        tabs_area: layout.tabs_area,
        category_area: layout.category_area,
    })
}

fn handle_overlay_scroll(
    view: &mut crate::ui::runtime_view::ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
    jump_rows: usize,
    kind: crossterm::event::MouseEventKind,
) -> bool {
    let delta = match kind {
        crossterm::event::MouseEventKind::ScrollUp => -SCROLL_STEP_I32,
        crossterm::event::MouseEventKind::ScrollDown => SCROLL_STEP_I32,
        _ => return false,
    };
    let areas = overlay_areas(layout);
    let counts = overlay_counts(ctx, jump_rows);
    let _ = with_active_table_handle(view, areas, counts, |mut handle| handle.scroll_by(delta));
    true
}

fn overlay_row_at(
    view: &mut crate::ui::runtime_view::ViewState,
    ctx: &DispatchContext<'_>,
    layout: LayoutContext,
    jump_rows: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    let areas = overlay_areas(layout);
    let counts = overlay_counts(ctx, jump_rows);
    with_active_table_handle(view, areas, counts, |handle| handle.row_at(mouse_x, mouse_y)).flatten()
}

fn overlay_areas(layout: LayoutContext) -> OverlayAreas {
    OverlayAreas {
        full: layout.size,
        msg: layout.msg_area,
    }
}

fn overlay_counts(ctx: &DispatchContext<'_>, jump_rows: usize) -> OverlayRowCounts {
    OverlayRowCounts {
        tabs: ctx.tabs.len(),
        jump: jump_rows,
        models: ctx.registry.models.len(),
        prompts: ctx.prompt_registry.prompts.len(),
        help: help_rows_len(),
    }
}
