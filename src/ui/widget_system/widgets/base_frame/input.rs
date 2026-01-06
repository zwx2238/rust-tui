use crate::ui::draw_input::{InputDrawParams, draw_input};
use crate::ui::runtime_events::{handle_key_event, handle_mouse_event, handle_paste_event};
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::state::Focus;
use crate::ui::widget_system::bindings::{bind_active_tab, bind_event};
use crate::ui::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;

use super::helpers::point_in_rect;

pub(super) struct InputWidget;

impl Widget for InputWidget {
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
        _jump_rows: &[crate::ui::jump::JumpRow],
        rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        match event {
            crossterm::event::Event::Mouse(m) => handle_input_mouse(ctx, layout, update, rect, *m),
            crossterm::event::Event::Key(key) => handle_input_key(ctx, layout, *key),
            crossterm::event::Event::Paste(paste) => handle_input_paste(ctx, paste),
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let theme = frame.state.theme;
        if let Some(app) = frame.state.active_app_mut() {
            draw_input(
                frame.frame,
                InputDrawParams {
                    area: rect,
                    input: &mut app.input,
                    theme,
                    focused: app.focus == Focus::Input,
                    busy: app.busy,
                    model_key: &app.model_key,
                    prompt_key: &app.prompt_key,
                },
            );
        }
        Ok(())
    }
}

fn handle_input_mouse(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    rect: ratatui::layout::Rect,
    m: crossterm::event::MouseEvent,
) -> Result<EventResult, Box<dyn Error>> {
    if !point_in_rect(m.column, m.row, rect) {
        return Ok(EventResult::ignored());
    }
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
    Ok(EventResult::handled())
}

fn handle_input_key(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    key: crossterm::event::KeyEvent,
) -> Result<EventResult, Box<dyn Error>> {
    if let Some(mut active) = bind_active_tab(ctx.tabs, *ctx.active_tab) {
        let app = active.app();
        if app.focus == Focus::Input && !app.busy {
            let handled = handle_key_event(
                key,
                ctx.tabs,
                *ctx.active_tab,
                ctx.args,
                layout.layout.msg_width,
                ctx.theme,
            )?;
            return Ok(if handled {
                EventResult::handled()
            } else {
                EventResult::ignored()
            });
        }
    }
    Ok(EventResult::ignored())
}

fn handle_input_paste(ctx: &mut EventCtx<'_>, paste: &str) -> Result<EventResult, Box<dyn Error>> {
    handle_paste_event(paste, ctx.tabs, *ctx.active_tab);
    Ok(EventResult::handled())
}
