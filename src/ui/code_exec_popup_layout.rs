use ratatui::layout::{Constraint, Layout, Rect};

const MIN_POPUP_WIDTH: u16 = 40;
const MIN_POPUP_HEIGHT: u16 = 8;
pub(crate) const OUTER_MARGIN: u16 = 2;

#[derive(Copy, Clone)]
pub(crate) struct CodeExecPopupLayout {
    pub(crate) popup: Rect,
    pub(crate) code_text_area: Rect,
    pub(crate) code_scrollbar_area: Rect,
    pub(crate) stdout_text_area: Rect,
    pub(crate) stdout_scrollbar_area: Rect,
    pub(crate) stderr_text_area: Rect,
    pub(crate) stderr_scrollbar_area: Rect,
    pub(crate) reason_input_area: Rect,
    pub(crate) approve_btn: Rect,
    pub(crate) deny_btn: Rect,
    pub(crate) stop_btn: Rect,
    pub(crate) exit_btn: Rect,
}

pub(crate) fn code_exec_popup_layout(area: Rect, with_reason: bool) -> CodeExecPopupLayout {
    let safe = safe_rect(area);
    let popup = popup_rect(safe);
    let inner = inset_rect(popup, 1);
    let (body, reason_area, actions_area) = split_inner(inner, with_reason);
    let (code_text_area, code_scrollbar_area, stdout_text_area, stdout_scrollbar_area, stderr_text_area, stderr_scrollbar_area) = split_body(body);
    let (approve_btn, deny_btn, stop_btn, exit_btn) = action_buttons(actions_area);
    CodeExecPopupLayout {
        popup,
        code_text_area,
        code_scrollbar_area,
        stdout_text_area,
        stdout_scrollbar_area,
        stderr_text_area,
        stderr_scrollbar_area,
        reason_input_area: reason_area,
        approve_btn,
        deny_btn,
        stop_btn,
        exit_btn,
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

fn split_inner(inner: Rect, with_reason: bool) -> (Rect, Rect, Rect) {
    let chunks = if with_reason {
        Layout::vertical([
            Constraint::Min(6),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner)
    } else {
        Layout::vertical([Constraint::Min(6), Constraint::Length(3)]).split(inner)
    };
    let body = chunks[0];
    let reason_area = if with_reason { chunks[1] } else { empty_rect() };
    let actions_area = if with_reason { chunks[2] } else { chunks[1] };
    (body, reason_area, actions_area)
}

fn split_body(body: Rect) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    let body_cols = Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ])
    .split(body);
    let (code_text_area, code_scrollbar_area) = text_and_scrollbar(body_cols[0]);
    let out_chunks = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(body_cols[1]);
    let (stdout_text_area, stdout_scrollbar_area) = text_and_scrollbar(out_chunks[0]);
    let (stderr_text_area, stderr_scrollbar_area) = text_and_scrollbar(out_chunks[1]);
    (code_text_area, code_scrollbar_area, stdout_text_area, stdout_scrollbar_area, stderr_text_area, stderr_scrollbar_area)
}

fn text_and_scrollbar(area: Rect) -> (Rect, Rect) {
    let text_area = Rect { x: area.x, y: area.y, width: area.width.saturating_sub(1), height: area.height };
    let scrollbar_area = Rect { x: area.x.saturating_add(area.width.saturating_sub(1)), y: area.y, width: 1, height: area.height };
    (text_area, scrollbar_area)
}

fn action_buttons(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let gap = 2u16;
    let btn_width = area.width.saturating_sub(gap).saturating_div(2).max(6);
    let approve_btn = Rect { x: area.x, y: area.y, width: btn_width, height: area.height };
    let deny_btn = Rect {
        x: area.x.saturating_add(btn_width + gap),
        y: area.y,
        width: area.width.saturating_sub(btn_width + gap).max(btn_width),
        height: area.height,
    };
    let stop_btn = Rect { x: area.x, y: area.y, width: area.width, height: area.height };
    let exit_btn = Rect { x: area.x, y: area.y, width: area.width, height: area.height };
    (approve_btn, deny_btn, stop_btn, exit_btn)
}

fn empty_rect() -> Rect {
    Rect { x: 0, y: 0, width: 0, height: 0 }
}
