use ratatui::layout::{Constraint, Layout, Rect};

const MIN_POPUP_WIDTH: u16 = 40;
const MIN_POPUP_HEIGHT: u16 = 8;
pub(crate) const OUTER_MARGIN: u16 = 2;

#[derive(Copy, Clone)]
pub(crate) struct FilePatchPopupLayout {
    pub(crate) popup: Rect,
    pub(crate) preview_area: Rect,
    pub(crate) preview_scrollbar_area: Rect,
    pub(crate) apply_btn: Rect,
    pub(crate) cancel_btn: Rect,
}

pub(crate) fn file_patch_popup_layout(area: Rect) -> FilePatchPopupLayout {
    let safe = safe_rect(area);
    let popup = popup_rect(safe);
    let inner = inset_rect(popup, 1);
    let (preview, actions_area) = split_inner(inner);
    let (preview_area, preview_scrollbar_area) = text_and_scrollbar(preview);
    let (apply_btn, cancel_btn) = action_buttons(actions_area);
    FilePatchPopupLayout { popup, preview_area, preview_scrollbar_area, apply_btn, cancel_btn }
}

fn safe_rect(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(OUTER_MARGIN),
        y: area.y.saturating_add(OUTER_MARGIN),
        width: area.width.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
        height: area.height.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
    }
}

fn popup_rect(safe: Rect) -> Rect {
    let width = (safe.width * 75 / 100)
        .max(MIN_POPUP_WIDTH)
        .min(safe.width.saturating_sub(2).max(MIN_POPUP_WIDTH));
    let height = (safe.height * 65 / 100)
        .max(MIN_POPUP_HEIGHT)
        .min(safe.height.saturating_sub(2).max(MIN_POPUP_HEIGHT));
    Rect {
        x: safe.x + (safe.width.saturating_sub(width)) / 2,
        y: safe.y + (safe.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

fn inset_rect(rect: Rect, inset: u16) -> Rect {
    Rect {
        x: rect.x.saturating_add(inset),
        y: rect.y.saturating_add(inset),
        width: rect.width.saturating_sub(inset.saturating_mul(2)),
        height: rect.height.saturating_sub(inset.saturating_mul(2)),
    }
}

fn split_inner(inner: Rect) -> (Rect, Rect) {
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).split(inner);
    (chunks[0], chunks[1])
}

fn text_and_scrollbar(area: Rect) -> (Rect, Rect) {
    let text_area = Rect { x: area.x, y: area.y, width: area.width.saturating_sub(1), height: area.height };
    let scrollbar_area = Rect { x: area.x.saturating_add(area.width.saturating_sub(1)), y: area.y, width: 1, height: area.height };
    (text_area, scrollbar_area)
}

fn action_buttons(area: Rect) -> (Rect, Rect) {
    let gap = 2u16;
    let btn_width = area.width.saturating_sub(gap).saturating_div(2).max(6);
    let apply_btn = Rect { x: area.x, y: area.y, width: btn_width, height: area.height };
    let cancel_btn = Rect {
        x: area.x.saturating_add(btn_width + gap),
        y: area.y,
        width: area.width.saturating_sub(btn_width + gap).max(btn_width),
        height: area.height,
    };
    (apply_btn, cancel_btn)
}
