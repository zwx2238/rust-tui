use crate::framework::widget_system::widgets::overlay_table::centered_area;
use ratatui::layout::Rect;

const POPUP_MAX_HEIGHT: u16 = 16;

pub fn model_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 70, rows, POPUP_MAX_HEIGHT)
}

// layout helpers are centralized in overlay_table

// selection color handled by overlay_table
