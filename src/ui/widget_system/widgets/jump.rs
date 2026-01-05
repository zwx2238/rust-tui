use crate::ui::jump::{JumpRow, build_jump_rows, max_preview_width};
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::{OverlayTableController, clamp_overlay_tables};

pub(crate) struct JumpWidget {
    _private: (),
}

impl JumpWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for JumpWidget {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _layout: &FrameLayout,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let binding = bind_event(ctx, layout, update);
        let mut controller = OverlayTableController {
            dispatch: binding.dispatch,
            layout: binding.layout,
            view: binding.view,
            jump_rows,
        };
        controller.handle_event(event)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        refresh_jump_rows(frame);
        clamp_overlay_tables(frame.view, frame.state, frame.jump_rows.len());
        draw_jump_layout(frame);
        draw_jump_table(frame);
        Ok(())
    }
}

fn refresh_jump_rows(frame: &mut WidgetFrame<'_, '_, '_, '_>) {
    frame.jump_rows.clear();
    if !frame.view.overlay.is(crate::ui::overlay::OverlayKind::Jump) {
        return;
    }
    let rows = frame
        .state
        .with_active_tab(|tab| {
            build_jump_rows(
                &tab.app.messages,
                frame.state.msg_width,
                max_preview_width(frame.state.msg_area),
                tab.app.pending_assistant,
            )
        })
        .unwrap_or_default();
    frame.jump_rows.extend(rows);
}

fn draw_jump_layout(frame: &mut WidgetFrame<'_, '_, '_, '_>) {
    crate::ui::jump::draw_jump_layout(
        frame.frame,
        crate::ui::jump::JumpLayoutParams {
            theme: frame.state.theme,
            tab_labels: frame.state.tab_labels,
            active_tab_pos: frame.state.active_tab_pos,
            categories: frame.state.categories,
            active_category: frame.state.active_category,
            header_note: frame.state.header_note,
            startup_text: frame.state.startup_text,
            header_area: frame.state.header_area,
            category_area: frame.state.category_area,
            tabs_area: frame.state.tabs_area,
            footer_area: frame.state.footer_area,
        },
    );
}

fn draw_jump_table(frame: &mut WidgetFrame<'_, '_, '_, '_>) {
    crate::ui::jump::draw_jump_table(
        frame.frame,
        frame.state.msg_area,
        frame.jump_rows,
        frame.view.jump.selected,
        frame.state.theme,
        frame.view.jump.scroll,
    );
}
