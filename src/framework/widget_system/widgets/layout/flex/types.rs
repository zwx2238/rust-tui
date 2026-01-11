use crate::framework::widget_system::runtime::runtime_loop_steps::FrameLayout;
use crate::framework::widget_system::box_constraints::BoxConstraints;
use crate::framework::widget_system::context::{
    EventCtx, LayoutCtx, UpdateCtx, UpdateOutput, WidgetFrame,
};
use crate::framework::widget_system::lifecycle::{EventResult, Widget};
use crate::framework::widget_system::widget_pod::WidgetPod;
use ratatui::layout::{Rect, Size};
use std::error::Error;

use super::dispatch::{MouseDispatch, dispatch_mouse_pair, dispatch_mouse_triple};
use super::measure::{container_size_from_measured, measure_pair, measure_triple};
use super::split::{split_pair, split_triple};

#[derive(Copy, Clone, Debug)]
pub(crate) enum FlexAxis {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum FlexParam {
    Fixed(u16),
    Flex(u16),
    Intrinsic,
}

pub(super) struct FlexChild<W: Widget> {
    pub(super) pod: WidgetPod<W>,
    pub(super) param: FlexParam,
    pub(super) measured_main: u16,
}

impl<W: Widget> FlexChild<W> {
    fn new(widget: W, param: FlexParam) -> Self {
        Self {
            pod: WidgetPod::new(widget),
            param,
            measured_main: 0,
        }
    }
}

pub(crate) struct Flex2<A: Widget, B: Widget> {
    axis: FlexAxis,
    a: FlexChild<A>,
    b: FlexChild<B>,
}

impl<A: Widget, B: Widget> Flex2<A, B> {
    pub(crate) fn new(axis: FlexAxis, a: (A, FlexParam), b: (B, FlexParam)) -> Self {
        Self {
            axis,
            a: FlexChild::new(a.0, a.1),
            b: FlexChild::new(b.0, b.1),
        }
    }

    pub(crate) fn a_rect(&self) -> Rect {
        self.a.pod.rect()
    }

    pub(crate) fn b_widget(&self) -> &B {
        self.b.pod.widget()
    }
}

pub(crate) struct Flex3<A: Widget, B: Widget, C: Widget> {
    axis: FlexAxis,
    a: FlexChild<A>,
    b: FlexChild<B>,
    c: FlexChild<C>,
}

impl<A: Widget, B: Widget, C: Widget> Flex3<A, B, C> {
    pub(crate) fn new(
        axis: FlexAxis,
        a: (A, FlexParam),
        b: (B, FlexParam),
        c: (C, FlexParam),
    ) -> Self {
        Self {
            axis,
            a: FlexChild::new(a.0, a.1),
            b: FlexChild::new(b.0, b.1),
            c: FlexChild::new(c.0, c.1),
        }
    }

    pub(crate) fn a_rect(&self) -> Rect {
        self.a.pod.rect()
    }

    pub(crate) fn c_rect(&self) -> Rect {
        self.c.pod.rect()
    }

    pub(crate) fn b_widget(&self) -> &B {
        self.b.pod.widget()
    }
}

impl<A: Widget, B: Widget> Widget for Flex2<A, B> {
    fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        let max = bc.max;
        let total = measure_pair(ctx, self.axis, max, &mut self.a, &mut self.b)?;
        Ok(bc.constrain(container_size_from_measured(self.axis, max, total)))
    }

    fn place(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &mut FrameLayout,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        let (r1, r2) = split_pair(self.axis, rect, self.a.measured_main);
        self.a.pod.place(ctx, layout, r1)?;
        self.b.pod.place(ctx, layout, r2)?;
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.a.pod.update(ctx, layout, update)?;
        self.b.pod.update(ctx, layout, update)?;
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        rect: Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        dispatch_mouse_pair(
            &mut self.a.pod,
            &mut self.b.pod,
            MouseDispatch {
                ctx,
                event,
                layout,
                update,
                rect,
            },
        )
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.a.pod.render(frame, layout, update)?;
        self.b.pod.render(frame, layout, update)?;
        Ok(())
    }
}

impl<A: Widget, B: Widget, C: Widget> Widget for Flex3<A, B, C> {
    fn measure(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        bc: BoxConstraints,
    ) -> Result<Size, Box<dyn Error>> {
        let max = bc.max;
        let total = measure_triple(ctx, self.axis, max, &mut self.a, &mut self.b, &mut self.c)?;
        Ok(bc.constrain(container_size_from_measured(self.axis, max, total)))
    }

    fn place(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        layout: &mut FrameLayout,
        rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        let (r1, r2, r3) =
            split_triple(self.axis, rect, self.a.measured_main, self.b.measured_main);
        self.a.pod.place(ctx, layout, r1)?;
        self.b.pod.place(ctx, layout, r2)?;
        self.c.pod.place(ctx, layout, r3)?;
        Ok(())
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
    ) -> Result<(), Box<dyn Error>> {
        self.a.pod.update(ctx, layout, update)?;
        self.b.pod.update(ctx, layout, update)?;
        self.c.pod.update(ctx, layout, update)?;
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        event: &crossterm::event::Event,
        layout: &FrameLayout,
        update: &UpdateOutput,
        rect: Rect,
    ) -> Result<EventResult, Box<dyn Error>> {
        dispatch_mouse_triple(
            &mut self.a.pod,
            &mut self.b.pod,
            &mut self.c.pod,
            MouseDispatch {
                ctx,
                event,
                layout,
                update,
                rect,
            },
        )
    }

    fn render(
        &mut self,
        frame: &mut WidgetFrame<'_, '_, '_, '_>,
        layout: &FrameLayout,
        update: &UpdateOutput,
        _rect: Rect,
    ) -> Result<(), Box<dyn Error>> {
        self.a.pod.render(frame, layout, update)?;
        self.b.pod.render(frame, layout, update)?;
        self.c.pod.render(frame, layout, update)?;
        Ok(())
    }
}
