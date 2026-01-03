use crate::ui::jump::JumpRow;
use crate::ui::runtime_dispatch::{handle_key_event_loop, handle_mouse_event_loop};
use crate::ui::command_suggestions::refresh_command_suggestions;
use crate::ui::state::Focus;
use crate::ui::runtime_loop_steps::FrameLayout;
use std::error::Error;

use super::super::context::{EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use super::super::bindings::{bind_active_tab, bind_event};
use super::super::events::poll_event;
use super::super::lifecycle::Widget;
use super::frame::FrameWidget;

pub(crate) struct ChatWidget {
    frame: FrameWidget,
}

impl ChatWidget {
    pub(crate) fn new() -> Self {
        Self { frame: FrameWidget }
    }
}

impl Widget for ChatWidget {
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>) -> Result<FrameLayout, Box<dyn Error>> {
        self.frame.layout(ctx)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
    ) -> Result<UpdateOutput, Box<dyn Error>> {
        self.frame.update(ctx, layout)
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

    fn render(&mut self, frame: &mut WidgetFrame<'_, '_, '_, '_>) -> Result<(), Box<dyn Error>> {
        crate::ui::overlay_render_base::render_chat_view(frame.ctx)
    }
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
