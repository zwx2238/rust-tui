use crate::ui::runtime_loop_steps::FrameLayout;
use crate::ui::widget_system::box_constraints::BoxConstraints;
use crate::ui::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::ui::widget_system::lifecycle::{EventResult, Widget};
use crate::ui::widget_system::widget_pod::WidgetPod;
use ratatui::layout::{Rect, Size};
use std::error::Error;

pub(crate) struct Stack2<A: Widget, B: Widget> {
    a: WidgetPod<A>,
    b: WidgetPod<B>,
}

impl<A: Widget, B: Widget> Stack2<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self {
        Self {
            a: WidgetPod::new(a),
            b: WidgetPod::new(b),
        }
    }
}

impl<A: Widget, B: Widget> Widget for Stack2<A, B> {
    fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        let sa = self.a.measure(ctx, bc)?;
        let sb = self.b.measure(ctx, bc)?;
        Ok(Size {
            width: sa.width.max(sb.width),
            height: sa.height.max(sb.height),
        })
    }

    fn place(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &mut FrameLayout,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.a.place(ctx, layout, rect)?;
        self.b.place(ctx, layout, rect)?;
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.a.update(ctx, layout, update)?;
        self.b.update(ctx, layout, update)?;
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        jump_rows: &[crate::ui::jump::JumpRow],
        rect: Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        let Some((x, y)) = mouse_pos(event) else {
            return Ok(EventResult::ignored());
        };
        if !point_in_rect(x, y, rect) {
            return Ok(EventResult::ignored());
        }
        let r2 = self.b.event(ctx, event, layout, update, jump_rows)?;
        if r2.handled || r2.quit {
            return Ok(r2);
        }
        self.a.event(ctx, event, layout, update, jump_rows)
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.a.render(frame, layout, update)?;
        self.b.render(frame, layout, update)?;
        Ok(())
    }
}

fn mouse_pos(event: &crossterm::event::Event) -> Option<(u16, u16)> {
    let crossterm::event::Event::Mouse(m) = event else {
        return None;
    };
    Some((m.column, m.row))
}

fn point_in_rect(x: u16, y: u16, rect: Rect) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
