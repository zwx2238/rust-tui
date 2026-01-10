use crate::framework::widget_system::widgets::overlay_table::centered_area;
use ratatui::layout::Rect;

const POPUP_MAX_HEIGHT: u16 = 24;

pub fn question_review_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 96, rows, POPUP_MAX_HEIGHT)
}

pub fn question_review_list_area(area: Rect, rows: usize) -> Rect {
    let popup = question_review_popup_area(area, rows);
    question_review_layout(popup).0
}

fn question_review_layout(area: Rect) -> (Rect, Rect) {
    let list_width = list_width(area.width);
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Length(list_width),
            ratatui::layout::Constraint::Min(10),
        ])
        .split(area);
    (chunks[0], chunks[1])
}

fn list_width(total: u16) -> u16 {
    let desired = total.saturating_mul(38) / 100;
    let max_list = total.saturating_sub(32).max(26);
    desired.clamp(26, max_list)
}
