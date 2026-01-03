use crate::ui::file_patch_popup::draw_file_patch_popup;
use crate::ui::file_patch_popup_layout::file_patch_popup_layout;
use crate::ui::file_patch_popup_text::patch_max_scroll;
use crate::ui::jump::JumpRow;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::super::bindings::bind_event;
use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::lifecycle::{EventResult, Widget};
use super::overlay_table::OverlayTableController;

pub(crate) struct FilePatchWidget {
    _private: (),
}

impl FilePatchWidget {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Widget for FilePatchWidget {
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
        match event {
            crossterm::event::Event::Mouse(m) => {
                let mut binding = bind_event(ctx, layout, update);
                let handled = crate::ui::runtime_dispatch::mouse_overlay::handle_file_patch_overlay_mouse(
                    *m,
                    &mut binding.dispatch,
                    binding.layout,
                    binding.view,
                );
                if handled {
                    return Ok(EventResult::handled());
                }
                Ok(EventResult::ignored())
            }
            crossterm::event::Event::Key(_) => {
                let mut binding = bind_event(ctx, layout, update);
                let mut controller = OverlayTableController {
                    dispatch: binding.dispatch,
                    layout: binding.layout,
                    view: binding.view,
                    jump_rows,
                };
                controller.handle_event(event)
            }
            _ => Ok(EventResult::ignored()),
        }
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        _layout: &FrameLayout,
        _update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(result) = frame
            .state
            .with_active_tab_mut(|tab_state| -> Result<(), Box<dyn Error>> {
            let Some(pending) = tab_state.app.pending_file_patch.clone() else {
                return Ok(());
            };
            let popup = file_patch_popup_layout(frame.frame.area());
            clamp_patch_scroll(frame.state.theme, tab_state, &pending, popup);
            draw_file_patch_popup(
                frame.frame,
                frame.frame.area(),
                &pending,
                tab_state.app.file_patch_scroll,
                tab_state.app.file_patch_hover,
                frame.state.theme,
            );
            Ok(())
        }) {
            result?;
        }
        Ok(())
    }
}

fn clamp_patch_scroll(
    theme: &crate::render::RenderTheme,
    tab_state: &mut crate::ui::runtime_helpers::TabState,
    pending: &crate::ui::state::PendingFilePatch,
    layout: crate::ui::file_patch_popup_layout::FilePatchPopupLayout,
) {
    let max_scroll = patch_max_scroll(
        &pending.preview,
        layout.preview_area.width,
        layout.preview_area.height,
        theme,
    );
    if tab_state.app.file_patch_scroll > max_scroll {
        tab_state.app.file_patch_scroll = max_scroll;
    }
}
