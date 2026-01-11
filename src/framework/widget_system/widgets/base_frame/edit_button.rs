use crate::framework::widget_system::draw::inner_area;
use crate::framework::widget_system::draw::layout::{PADDING_X, PADDING_Y};
use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::bindings::bind_event;
use crate::framework::widget_system::context::{EventCtx, UpdateCtx, UpdateOutput, WidgetFrame};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use ratatui::style::{Color, Modifier, Style};
use std::error::Error;

use super::super::button::ButtonWidget;

pub(super) struct EditButtonWidget {
    buttons: Vec<EditButtonEntry>,
}

struct EditButtonEntry {
    msg_index: usize,
    button: ButtonWidget,
}

impl EditButtonWidget {
    pub(super) fn new() -> Self {
        Self {
            buttons: Vec::new(),
        }
    }

    fn sync_buttons(
        &mut self,
        layouts: &[crate::render::MessageLayout],
        scroll: u16,
        msg_area: ratatui::layout::Rect,
    ) {
        let inner = inner_area(msg_area, PADDING_X, PADDING_Y);
        let mut count = 0;
        for layout in layouts {
            let Some(rect) = button_rect(layout, scroll, inner) else {
                continue;
            };
            let entry = self.ensure_entry(count, layout.index);
            update_button_entry(entry, layout.index, rect);
            count += 1;
        }
        hide_unused_buttons(&mut self.buttons, count);
    }

    fn ensure_entry(&mut self, idx: usize, msg_index: usize) -> &mut EditButtonEntry {
        if self.buttons.len() <= idx {
            self.buttons.push(EditButtonEntry {
                msg_index,
                button: ButtonWidget::new("[编辑]"),
            });
        }
        &mut self.buttons[idx]
    }
}

impl Widget for EditButtonWidget {
    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        _update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(tab_state) = ctx.tabs.get(*ctx.active_tab) {
            self.sync_buttons(
                &tab_state.app.message_layouts,
                tab_state.app.scroll,
                layout.layout.msg_area,
            );
        }
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _jump_rows: &[crate::framework::widget_system::widgets::jump::JumpRow],
        _rect: ratatui::layout::Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let crossterm::event::Event::Mouse(m) = event else {
            return Ok(EventResult::ignored());
        };
        if !matches!(m.kind, crossterm::event::MouseEventKind::Down(_)) {
            return Ok(EventResult::ignored());
        }
        for entry in &mut self.buttons {
            if entry
                .button
                .event(ctx, event, layout, update, &[], layout.layout.msg_area)?
                .handled
            {
                fork_message(ctx, layout, update, entry.msg_index);
                return Ok(EventResult::handled());
            }
        }
        Ok(EventResult::ignored())
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: ratatui::layout::Rect,
    ) -> Result<(), Box<dyn Error>> {
        for entry in &mut self.buttons {
            let _ = entry
                .button
                .render(frame, layout, update, layout.layout.msg_area);
        }
        Ok(())
    }
}

fn button_rect(
    layout: &crate::render::MessageLayout,
    scroll: u16,
    inner: ratatui::layout::Rect,
) -> Option<ratatui::layout::Rect> {
    let (start, end) = layout.button_range?;
    let row = layout.label_line as i32 - scroll as i32;
    if row < 0 || row >= inner.height as i32 {
        return None;
    }
    let start_u = start as u16;
    if start_u >= inner.width {
        return None;
    }
    let end_u = (end as u16).min(inner.width);
    let width = end_u.saturating_sub(start_u);
    if width == 0 {
        return None;
    }
    Some(ratatui::layout::Rect {
        x: inner.x + start_u,
        y: inner.y + row as u16,
        width,
        height: 1,
    })
}

fn update_button_entry(entry: &mut EditButtonEntry, msg_index: usize, rect: ratatui::layout::Rect) {
    entry.msg_index = msg_index;
    entry.button.set_rect(rect);
    entry.button.set_label("[编辑]");
    entry.button.set_visible(true);
    entry.button.set_bordered(false);
    entry.button.set_style(edit_button_style());
}

fn hide_unused_buttons(entries: &mut [EditButtonEntry], used: usize) {
    for entry in entries.iter_mut().skip(used) {
        entry.button.set_visible(false);
    }
}

fn fork_message(
    ctx: &mut EventCtx<'_>,
    layout: &FrameLayout,
    update: &UpdateOutput,
    msg_index: usize,
) {
    let mut binding = bind_event(ctx, layout, update);
    let _ =
        crate::framework::widget_system::runtime_dispatch::fork::fork_message_by_index(&mut binding.dispatch, msg_index);
}

fn edit_button_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}
