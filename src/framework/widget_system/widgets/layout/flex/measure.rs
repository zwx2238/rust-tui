use crate::framework::widget_system::box_constraints::BoxConstraints;
use ratatui::layout::Size;
use std::error::Error;

use super::types::{FlexAxis, FlexChild, FlexParam};
use crate::framework::widget_system::context::LayoutCtx;
use crate::framework::widget_system::lifecycle::Widget;

pub(super) fn measure_pair<A: Widget, B: Widget>(
    ctx: &mut LayoutCtx<'_>,
    axis: FlexAxis,
    max: Size,
    a: &mut FlexChild<A>,
    b: &mut FlexChild<B>,
) -> Result<u16, Box<dyn Error>> {
    let used = measure_non_flex(ctx, axis, max, a, 0)?;
    let used = measure_non_flex(ctx, axis, max, b, used)?;
    let remaining = remaining_main(axis, max, used);
    let total_weight = flex_weight(a.param).saturating_add(flex_weight(b.param));
    let (fa, fb) = allocate_flex2(
        remaining,
        total_weight,
        flex_weight(a.param),
        flex_weight(b.param),
    );
    measure_flex(ctx, axis, max, a, fa)?;
    measure_flex(ctx, axis, max, b, fb)?;
    Ok(used.saturating_add(fa).saturating_add(fb))
}

pub(super) fn measure_triple<A: Widget, B: Widget, C: Widget>(
    ctx: &mut LayoutCtx<'_>,
    axis: FlexAxis,
    max: Size,
    a: &mut FlexChild<A>,
    b: &mut FlexChild<B>,
    c: &mut FlexChild<C>,
) -> Result<u16, Box<dyn Error>> {
    let used = measure_non_flex(ctx, axis, max, a, 0)?;
    let used = measure_non_flex(ctx, axis, max, b, used)?;
    let used = measure_non_flex(ctx, axis, max, c, used)?;
    let remaining = remaining_main(axis, max, used);
    let total_weight = flex_weight(a.param)
        .saturating_add(flex_weight(b.param))
        .saturating_add(flex_weight(c.param));
    let (fa, fb, fc) = allocate_flex3(
        remaining,
        total_weight,
        flex_weight(a.param),
        flex_weight(b.param),
        flex_weight(c.param),
    );
    measure_flex(ctx, axis, max, a, fa)?;
    measure_flex(ctx, axis, max, b, fb)?;
    measure_flex(ctx, axis, max, c, fc)?;
    Ok(used
        .saturating_add(fa)
        .saturating_add(fb)
        .saturating_add(fc))
}

pub(super) fn container_size_from_measured(axis: FlexAxis, max: Size, total_main: u16) -> Size {
    match axis {
        FlexAxis::Horizontal => Size {
            width: total_main.min(max.width),
            height: max.height,
        },
        FlexAxis::Vertical => Size {
            width: max.width,
            height: total_main.min(max.height),
        },
    }
}

fn measure_non_flex<W: Widget>(
    ctx: &mut LayoutCtx<'_>,
    axis: FlexAxis,
    max: Size,
    child: &mut FlexChild<W>,
    used: u16,
) -> Result<u16, Box<dyn Error>> {
    match child.param {
        FlexParam::Fixed(v) => {
            let v = clamp_main(axis, v, max);
            child.measured_main = v;
            let _ = child.pod.measure(ctx, tight_for(axis, max, v))?;
            Ok(used.saturating_add(v))
        }
        FlexParam::Intrinsic => {
            let size = child.pod.measure(ctx, BoxConstraints::loose(max))?;
            let v = clamp_main(axis, main(axis, size), max);
            child.measured_main = v;
            Ok(used.saturating_add(v))
        }
        FlexParam::Flex(_) => Ok(used),
    }
}

fn measure_flex<W: Widget>(
    ctx: &mut LayoutCtx<'_>,
    axis: FlexAxis,
    max: Size,
    child: &mut FlexChild<W>,
    alloc: u16,
) -> Result<(), Box<dyn Error>> {
    if !matches!(child.param, FlexParam::Flex(_)) {
        return Ok(());
    }
    child.measured_main = clamp_main(axis, alloc, max);
    let _ = child
        .pod
        .measure(ctx, tight_for(axis, max, child.measured_main))?;
    Ok(())
}

fn flex_weight(param: FlexParam) -> u16 {
    match param {
        FlexParam::Flex(w) => w,
        _ => 0,
    }
}

fn allocate_flex2(remaining: u16, total: u16, w1: u16, _w2: u16) -> (u16, u16) {
    if total == 0 {
        return (0, 0);
    }
    let a = remaining.saturating_mul(w1) / total;
    let b = remaining.saturating_sub(a);
    (a, b)
}

fn allocate_flex3(remaining: u16, total: u16, w1: u16, w2: u16, _w3: u16) -> (u16, u16, u16) {
    if total == 0 {
        return (0, 0, 0);
    }
    let a = remaining.saturating_mul(w1) / total;
    let b = remaining.saturating_mul(w2) / total;
    let c = remaining.saturating_sub(a.saturating_add(b));
    (a, b, c)
}

fn main(axis: FlexAxis, size: Size) -> u16 {
    match axis {
        FlexAxis::Horizontal => size.width,
        FlexAxis::Vertical => size.height,
    }
}

fn clamp_main(axis: FlexAxis, v: u16, max: Size) -> u16 {
    match axis {
        FlexAxis::Horizontal => v.min(max.width),
        FlexAxis::Vertical => v.min(max.height),
    }
}

fn remaining_main(axis: FlexAxis, max: Size, used: u16) -> u16 {
    match axis {
        FlexAxis::Horizontal => max.width.saturating_sub(used),
        FlexAxis::Vertical => max.height.saturating_sub(used),
    }
}

fn tight_for(axis: FlexAxis, max: Size, main: u16) -> BoxConstraints {
    match axis {
        FlexAxis::Horizontal => BoxConstraints::tight(Size {
            width: main,
            height: max.height,
        }),
        FlexAxis::Vertical => BoxConstraints::tight(Size {
            width: max.width,
            height: main,
        }),
    }
}
