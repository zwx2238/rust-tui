use ratatui::layout::Rect;

const MIN_POPUP_WIDTH: u16 = 40;
const MIN_POPUP_HEIGHT: u16 = 10;
const OUTER_MARGIN: u16 = 2;

#[derive(Copy, Clone)]
pub(crate) struct TerminalPopupLayout {
    pub(crate) popup: Rect,
    pub(crate) terminal_area: Rect,
}

pub(crate) fn terminal_popup_layout(area: Rect) -> TerminalPopupLayout {
    let safe = safe_rect(area);
    let popup = popup_rect(safe);
    let terminal_area = inset_rect(popup, 1);
    TerminalPopupLayout {
        popup,
        terminal_area,
    }
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
    let width = (safe.width * 85 / 100)
        .max(MIN_POPUP_WIDTH)
        .min(safe.width.saturating_sub(2).max(MIN_POPUP_WIDTH));
    let height = (safe.height * 70 / 100)
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
