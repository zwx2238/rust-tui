use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub const PADDING_X: u16 = 1;
pub const PADDING_Y: u16 = 0;
pub const SCROLLBAR_WIDTH: u16 = 2;

pub fn layout_chunks(size: Rect, input_height: u16) -> (Rect, Rect, Rect, Rect, Rect) {
    let input_constraint = if input_height == 0 {
        Constraint::Length(0)
    } else {
        Constraint::Length(input_height)
    };
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(3),
                input_constraint,
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(size);
    (layout[0], layout[1], layout[2], layout[3], layout[4])
}

pub fn inner_area(area: Rect, padding_x: u16, padding_y: u16) -> Rect {
    Rect {
        x: area.x + 1 + padding_x,
        y: area.y + 1 + padding_y,
        width: area.width.saturating_sub(2 + padding_x * 2),
        height: area.height.saturating_sub(2 + padding_y * 2),
    }
}

pub fn input_inner_area(area: Rect) -> Rect {
    inner_area(area, PADDING_X, PADDING_Y)
}

pub fn inner_width(area: Rect, padding_x: u16) -> usize {
    area.width.saturating_sub(2 + padding_x * 2) as usize
}

pub fn inner_height(area: Rect, padding_y: u16) -> u16 {
    area.height.saturating_sub(2 + padding_y * 2)
}

pub fn scrollbar_area(area: Rect) -> Rect {
    let width = SCROLLBAR_WIDTH.min(area.width);
    Rect {
        x: area.x.saturating_add(area.width.saturating_sub(width)),
        y: area.y.saturating_add(1),
        width,
        height: area.height.saturating_sub(2),
    }
}
