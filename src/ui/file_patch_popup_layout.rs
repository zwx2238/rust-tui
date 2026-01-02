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
    let safe = Rect {
        x: area.x.saturating_add(OUTER_MARGIN),
        y: area.y.saturating_add(OUTER_MARGIN),
        width: area.width.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
        height: area.height.saturating_sub(OUTER_MARGIN.saturating_mul(2)),
    };
    let width = (safe.width * 75 / 100)
        .max(MIN_POPUP_WIDTH)
        .min(safe.width.saturating_sub(2).max(MIN_POPUP_WIDTH));
    let height = (safe.height * 65 / 100)
        .max(MIN_POPUP_HEIGHT)
        .min(safe.height.saturating_sub(2).max(MIN_POPUP_HEIGHT));
    let x = safe.x + (safe.width.saturating_sub(width)) / 2;
    let y = safe.y + (safe.height.saturating_sub(height)) / 2;
    let popup = Rect { x, y, width, height };
    let inner = Rect {
        x: popup.x.saturating_add(1),
        y: popup.y.saturating_add(1),
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).split(inner);
    let preview = chunks[0];
    let actions_area = chunks[1];
    let preview_area = Rect {
        x: preview.x,
        y: preview.y,
        width: preview.width.saturating_sub(1),
        height: preview.height,
    };
    let preview_scrollbar_area = Rect {
        x: preview.x.saturating_add(preview.width.saturating_sub(1)),
        y: preview.y,
        width: 1,
        height: preview.height,
    };
    let gap = 2u16;
    let btn_width = actions_area
        .width
        .saturating_sub(gap)
        .saturating_div(2)
        .max(6);
    let apply_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: btn_width,
        height: actions_area.height,
    };
    let cancel_btn = Rect {
        x: actions_area.x.saturating_add(btn_width + gap),
        y: actions_area.y,
        width: actions_area
            .width
            .saturating_sub(btn_width + gap)
            .max(btn_width),
        height: actions_area.height,
    };
    FilePatchPopupLayout {
        popup,
        preview_area,
        preview_scrollbar_area,
        apply_btn,
        cancel_btn,
    }
}
