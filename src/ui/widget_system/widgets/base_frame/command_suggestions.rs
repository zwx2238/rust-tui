use crate::ui::command_suggestions::{draw_command_suggestions, handle_command_suggestion_click};
use crate::ui::widget_system::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

pub(super) struct CommandSuggestionsWidget;

impl Widget for CommandSuggestionsWidget {
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
        _update: &UpdateOutput,
        _jump_rows: &[crate::ui::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
            return Ok(EventResult::ignored());
        }
        if let Some(tab_state) = ctx.tabs.get_mut(*ctx.active_tab)
            && handle_command_suggestion_click(
                &mut tab_state.app,
                layout.layout.msg_area,
                layout.layout.input_area,
                m.column,
                m.row,
            )
        {
            return Ok(EventResult::handled());
        }
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let msg_area = frame.state.msg_area;
        let input_area = frame.state.input_area;
        let theme = frame.state.theme;
        if let Some(app) = frame.state.active_app_mut() {
            draw_command_suggestions(
                frame.frame,
                msg_area,
                input_area,
                app,
                theme,
            );
        }
        Ok(())
    }
}
