use crate::framework::widget_system::events::{handle_key_event, handle_paste_event};
use crate::framework::widget_system::bindings::bind_active_tab;
use crate::framework::widget_system::context::{EventCtx, UpdateOutput};
use crate::framework::widget_system::lifecycle::EventResult;
use crate::ui::{jump::JumpRow, runtime_loop_steps::FrameLayout};
use std::error::Error;

use super::base::BaseFrameWidget;
use super::helpers::pod_event_handled;

impl BaseFrameWidget {
    pub(super) fn handle_key(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        key: crossterm::event::KeyEvent,
    ) -> Result<EventResult, Box<dyn Error>> {
        if let Some(result) = self.handle_global_key(ctx, layout, update, jump_rows, key)? {
            return Ok(result);
        }
        self.handle_active_tab_key(ctx, layout, key)
    }

    fn handle_global_key(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        key: crossterm::event::KeyEvent,
    ) -> Result<Option<EventResult>, Box<dyn Error>> {
        let result = self.global_keys.event(
            ctx,
            &crossterm::event::Event::Key(key),
            layout,
            update,
            jump_rows,
        )?;
        Ok(if result.handled || result.quit {
            Some(result)
        } else {
            None
        })
    }

    fn handle_active_tab_key(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        key: crossterm::event::KeyEvent,
    ) -> Result<EventResult, Box<dyn Error>> {
        if let Some(mut active) = bind_active_tab(ctx.tabs, *ctx.active_tab) {
            let app = active.app();
            if !app.busy {
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

    pub(super) fn handle_paste(
        &mut self,
        ctx: &mut EventCtx<'_>,
        paste: &str,
    ) -> Result<EventResult, Box<dyn Error>> {
        handle_paste_event(paste, ctx.tabs, *ctx.active_tab);
        Ok(EventResult::handled())
    }

    pub(super) fn handle_mouse(
        &mut self,
        ctx: &mut EventCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[JumpRow],
        m: crossterm::event::MouseEvent,
    ) -> Result<EventResult, Box<dyn Error>> {
        if pod_event_handled(
            &mut self.command_suggestions,
            ctx,
            &crossterm::event::Event::Mouse(m),
            layout,
            update,
            jump_rows,
        )? {
            return Ok(EventResult::handled());
        }
        self.root.event(
            ctx,
            &crossterm::event::Event::Mouse(m),
            layout,
            update,
            jump_rows,
        )
    }
}
