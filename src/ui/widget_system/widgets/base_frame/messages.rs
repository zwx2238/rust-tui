use crate::ui::draw::{MessagesDrawParams, draw_messages};
use crate::ui::runtime_events::handle_mouse_event;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::state::Focus;
use crate::ui::widget_system::bindings::bind_event;
use crate::ui::widget_system::context::{
    EventCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;

use super::helpers::{point_in_rect, scrollbar_hit};

pub(super) struct MessagesWidget;

impl Widget for MessagesWidget {
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
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !point_in_rect(m.column, m.row, rect)
            && !scrollbar_hit(layout.layout.msg_area, m.column, m.row)
        {
            return Ok(EventResult::ignored());
        }
        handle_messages_mouse(ctx, layout, update, *m);
        Ok(EventResult::handled())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(app) = frame.state.active_app() {
            draw_messages(
                frame.frame,
                MessagesDrawParams {
                    area: rect,
                    text: frame.state.text,
                    scroll: app.scroll,
                    theme: frame.state.theme,
                    focused: app.focus == Focus::Chat,
                    total_lines: frame.state.total_lines,
                    selection: app.chat_selection,
                },
            );
        }
        Ok(())
    }
}

fn handle_messages_mouse(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    m: crossterm::event::MouseEvent,
) {
    let binding = bind_event(ctx, layout, update);
    let _ = handle_mouse_event(crate::ui::runtime_events::MouseEventParams {
        m,
        tabs: binding.dispatch.tabs,
        active_tab: binding.dispatch.active_tab,
        categories: binding.dispatch.categories,
        active_category: binding.dispatch.active_category,
        tabs_area: binding.layout.tabs_area,
        msg_area: binding.layout.msg_area,
        input_area: binding.layout.input_area,
        category_area: binding.layout.category_area,
        msg_width: binding.dispatch.msg_width,
        view_height: binding.layout.view_height,
        total_lines: update.active_data.total_lines,
        theme: binding.dispatch.theme,
    });
}
