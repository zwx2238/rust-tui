use crate::framework::widget_system::widgets::overlay_table::centered_area;
use ratatui::layout::Rect;

const POPUP_MAX_HEIGHT: u16 = 18;

pub fn prompt_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 80, rows, POPUP_MAX_HEIGHT)
}
