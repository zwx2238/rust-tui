use ratatui::layout::Rect;

pub fn popup_area(area: Rect, percent_x: u16, rows: usize, max_height: u16) -> Rect {
    let body = rows.max(1) as u16;
    let height = (body + 3).min(max_height);
    centered_rect(area, percent_x, height)
}

fn centered_rect(area: Rect, percent_x: u16, height: u16) -> Rect {
    let width = area.width * percent_x / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let h = height.min(area.height.saturating_sub(2)).max(3);
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width,
        height: h,
    }
}
