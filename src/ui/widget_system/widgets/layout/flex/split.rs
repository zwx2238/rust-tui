use ratatui::layout::Rect;

use super::types::FlexAxis;

pub(super) fn split_pair(axis: FlexAxis, rect: Rect, a_main: u16) -> (Rect, Rect) {
    match axis {
        FlexAxis::Horizontal => split_pair_h(rect, a_main),
        FlexAxis::Vertical => split_pair_v(rect, a_main),
    }
}

pub(super) fn split_triple(
    axis: FlexAxis,
    rect: Rect,
    a_main: u16,
    b_main: u16,
) -> (Rect, Rect, Rect) {
    match axis {
        FlexAxis::Horizontal => split_triple_h(rect, a_main, b_main),
        FlexAxis::Vertical => split_triple_v(rect, a_main, b_main),
    }
}

fn split_pair_h(rect: Rect, w1: u16) -> (Rect, Rect) {
    let w1 = w1.min(rect.width);
    (
        Rect {
            x: rect.x,
            y: rect.y,
            width: w1,
            height: rect.height,
        },
        Rect {
            x: rect.x.saturating_add(w1),
            y: rect.y,
            width: rect.width.saturating_sub(w1),
            height: rect.height,
        },
    )
}

fn split_pair_v(rect: Rect, h1: u16) -> (Rect, Rect) {
    let h1 = h1.min(rect.height);
    (
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: h1,
        },
        Rect {
            x: rect.x,
            y: rect.y.saturating_add(h1),
            width: rect.width,
            height: rect.height.saturating_sub(h1),
        },
    )
}

fn split_triple_h(rect: Rect, w1: u16, w2: u16) -> (Rect, Rect, Rect) {
    let w1 = w1.min(rect.width);
    let w2 = w2.min(rect.width.saturating_sub(w1));
    (
        Rect {
            x: rect.x,
            y: rect.y,
            width: w1,
            height: rect.height,
        },
        Rect {
            x: rect.x.saturating_add(w1),
            y: rect.y,
            width: w2,
            height: rect.height,
        },
        Rect {
            x: rect.x.saturating_add(w1.saturating_add(w2)),
            y: rect.y,
            width: rect.width.saturating_sub(w1.saturating_add(w2)),
            height: rect.height,
        },
    )
}

fn split_triple_v(rect: Rect, h1: u16, h2: u16) -> (Rect, Rect, Rect) {
    let h1 = h1.min(rect.height);
    let h2 = h2.min(rect.height.saturating_sub(h1));
    (
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: h1,
        },
        Rect {
            x: rect.x,
            y: rect.y.saturating_add(h1),
            width: rect.width,
            height: h2,
        },
        Rect {
            x: rect.x,
            y: rect.y.saturating_add(h1.saturating_add(h2)),
            width: rect.width,
            height: rect.height.saturating_sub(h1.saturating_add(h2)),
        },
    )
}
