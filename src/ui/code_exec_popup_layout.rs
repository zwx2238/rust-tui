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
    let reason_area = if with_reason {
        chunks[1]
    } else {
        Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    };
    let actions_area = if with_reason { chunks[2] } else { chunks[1] };
    let body_cols = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(body);
    let code_text_area = Rect {
        x: body_cols[0].x,
        y: body_cols[0].y,
        width: body_cols[0].width.saturating_sub(1),
        height: body_cols[0].height,
    };
    let code_scrollbar_area = Rect {
        x: body_cols[0].x.saturating_add(body_cols[0].width.saturating_sub(1)),
        y: body_cols[0].y,
        width: 1,
        height: body_cols[0].height,
    };
    let out_chunks = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(body_cols[1]);
    let stdout_text_area = Rect {
        x: out_chunks[0].x,
        y: out_chunks[0].y,
        width: out_chunks[0].width.saturating_sub(1),
        height: out_chunks[0].height,
    };
    let stdout_scrollbar_area = Rect {
        x: out_chunks[0].x.saturating_add(out_chunks[0].width.saturating_sub(1)),
        y: out_chunks[0].y,
        width: 1,
        height: out_chunks[0].height,
    };
    let stderr_text_area = Rect {
        x: out_chunks[1].x,
        y: out_chunks[1].y,
        width: out_chunks[1].width.saturating_sub(1),
        height: out_chunks[1].height,
    };
    let stderr_scrollbar_area = Rect {
        x: out_chunks[1].x.saturating_add(out_chunks[1].width.saturating_sub(1)),
        y: out_chunks[1].y,
        width: 1,
        height: out_chunks[1].height,
    };
    let gap = 2u16;
    let btn_width = actions_area
        .width
        .saturating_sub(gap)
        .saturating_div(2)
        .max(6);
    let approve_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: btn_width,
        height: actions_area.height,
    };
    let deny_btn = Rect {
        x: actions_area.x.saturating_add(btn_width + gap),
        y: actions_area.y,
        width: actions_area
            .width
            .saturating_sub(btn_width + gap)
            .max(btn_width),
        height: actions_area.height,
    };
    let stop_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: actions_area.width,
        height: actions_area.height,
    };
    let exit_btn = Rect {
        x: actions_area.x,
        y: actions_area.y,
        width: actions_area.width,
        height: actions_area.height,
    };
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
