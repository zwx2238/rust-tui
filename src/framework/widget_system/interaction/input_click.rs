use crate::framework::widget_system::draw::input_inner_area;
use crate::framework::widget_system::runtime::state::App;
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthChar;

pub fn update_input_view_top(app: &mut App, input_area: Rect) {
    let inner = input_inner_area(input_area);
    let height = inner.height.max(1) as u16;
    let cursor_row = app.input.cursor().0 as u16;
    let prev_top = app.input_view_top_row;
    app.input_view_top_row = next_scroll_top(prev_top, cursor_row, height);
}

pub fn click_to_cursor(app: &App, input_area: Rect, column: u16, row: u16) -> (usize, usize) {
    let inner = input_inner_area(input_area);
    if inner.width == 0 || inner.height == 0 {
        return end_cursor(app);
    }
    if column < inner.x || row < inner.y {
        return end_cursor(app);
    }
    let rel_row = row.saturating_sub(inner.y);
    let rel_col = column.saturating_sub(inner.x);
    let rel_row = rel_row.min(inner.height.saturating_sub(1));
    let rel_col = rel_col.min(inner.width.saturating_sub(1));

    let abs_row = app.input_view_top_row.saturating_add(rel_row) as usize;
    let lines = app.input.lines();
    if lines.is_empty() {
        return (0, 0);
    }
    let row_idx = abs_row.min(lines.len() - 1);
    let col_idx = column_to_char_idx(&lines[row_idx], rel_col as usize);
    (row_idx, col_idx)
}

fn next_scroll_top(prev_top: u16, cursor: u16, len: u16) -> u16 {
    if cursor < prev_top {
        cursor
    } else if prev_top + len <= cursor {
        cursor + 1 - len
    } else {
        prev_top
    }
}

fn column_to_char_idx(line: &str, target_col: usize) -> usize {
    let mut col = 0usize;
    let mut idx = 0usize;
    for ch in line.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);
        if target_col < col + w {
            return idx;
        }
        col += w;
        idx += 1;
    }
    idx
}

fn end_cursor(app: &App) -> (usize, usize) {
    let lines = app.input.lines();
    if lines.is_empty() {
        return (0, 0);
    }
    let row = lines.len() - 1;
    let col = lines[row].chars().count();
    (row, col)
}
