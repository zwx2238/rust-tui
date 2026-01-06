use crate::ui::file_patch_popup::draw_file_patch_popup_base;
use crate::ui::file_patch_popup_layout::file_patch_popup_layout;
use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::context::{
    EventCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use std::error::Error;

use super::buttons::render_buttons;
use super::event::{handle_key_event, handle_mouse_event};
use super::selection::clamp_patch_scroll;

pub(crate) struct FilePatchWidget {
    pub(super) apply_btn: crate::ui::widget_system::widgets::button::ButtonWidget,
    pub(super) cancel_btn: crate::ui::widget_system::widgets::button::ButtonWidget,
}

impl FilePatchWidget {
    pub(crate) fn new() -> Self {
        Self {
            apply_btn: crate::ui::widget_system::widgets::button::ButtonWidget::new("应用修改"),
            cancel_btn: crate::ui::widget_system::widgets::button::ButtonWidget::new("取消"),
        }
    }
}

impl Widget for FilePatchWidget {
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
        match event {
            crossterm::event::Event::Mouse(m) => handle_mouse_event(self, ctx, layout, update, *m),
            crossterm::event::Event::Key(_) => {
                handle_key_event(ctx, layout, update, jump_rows, event)
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        let hover = render_popup(frame, rect)?;
        render_buttons(
            self,
            rect,
            hover,
            frame.state.theme,
            frame,
            layout,
            update,
        );
        Ok(())
    }
}

fn render_popup(
    frame: &mut WidgetFrame<'_, '_, '_, '_>,
    rect: ratatui::layout::Rect,
) -> Result<Option<crate::ui::state::FilePatchHover>, Box<dyn Error>> {
    let active_tab = frame.state.active_tab;
    let Some(tab_state) = frame.state.tabs.get_mut(active_tab) else {
        return Ok(None);
    };
    let Some(pending) = tab_state.app.pending_file_patch.clone() else {
        return Ok(None);
    };
    let popup = file_patch_popup_layout(rect);
    clamp_patch_scroll(frame.state.theme, tab_state, &pending, popup);
    draw_file_patch_popup_base(
        frame.frame,
        rect,
        &pending,
        tab_state.app.file_patch_scroll,
        tab_state.app.file_patch_selection,
        frame.state.theme,
    );
    Ok(tab_state.app.file_patch_hover)
}
