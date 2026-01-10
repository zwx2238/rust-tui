use crate::framework::widget_system::interaction::scroll::max_scroll_u16;

pub(crate) fn point_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

pub(crate) fn scroll_from_mouse(
    total_lines: usize,
    view_height: u16,
    scroll_area: ratatui::layout::Rect,
    mouse_y: u16,
) -> u16 {
    if total_lines <= view_height as usize || scroll_area.height <= 1 {
        return 0;
    }
    let max_scroll = max_scroll_u16(total_lines, view_height);
    let y = mouse_y.saturating_sub(scroll_area.y);
    let track = scroll_area.height.saturating_sub(1).max(1);
    let ratio = y.min(track) as f32 / track as f32;
    let scroll = (ratio * max_scroll as f32).round() as u16;
    scroll.min(max_scroll)
}
