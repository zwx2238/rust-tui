use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub const PADDING_X: u16 = 1;
pub const PADDING_Y: u16 = 0;
pub const SCROLLBAR_WIDTH: u16 = 2;

pub fn layout_chunks(
    size: Rect,
    input_height: u16,
    sidebar_width: u16,
) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    let input_constraint = input_constraint(input_height);
    let vertical = split_vertical(size);
    let body = vertical[1];
    let horizontal = split_horizontal(body, sidebar_width);
    let sidebar_area = horizontal[0];
    let main = horizontal[1];
    let main_split = split_main(main, input_constraint);
    (
        vertical[0],
        sidebar_area,
        main_split[0],
        main_split[1],
        main_split[2],
        vertical[2],
    )
}

fn input_constraint(input_height: u16) -> Constraint {
    if input_height == 0 {
        Constraint::Length(0)
    } else {
        Constraint::Length(input_height)
    }
}

fn split_vertical(size: Rect) -> std::vec::Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(size)
        .to_vec()
}

fn split_horizontal(body: Rect, sidebar_width: u16) -> std::vec::Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_width), Constraint::Min(10)].as_ref())
        .split(body)
        .to_vec()
}

fn split_main(main: Rect, input_constraint: Constraint) -> std::vec::Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3), input_constraint].as_ref())
        .split(main)
        .to_vec()
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
