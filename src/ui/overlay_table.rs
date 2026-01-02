use crate::render::RenderTheme;
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, Table, TableState};

pub struct OverlayTable<'a> {
    pub title: Line<'a>,
    pub header: Row<'a>,
    pub rows: Vec<Row<'a>>,
    pub widths: Vec<Constraint>,
    pub selected: usize,
    pub scroll: usize,
    pub theme: &'a RenderTheme,
}

pub fn draw_overlay_table(f: &mut ratatui::Frame<'_>, area: Rect, table: OverlayTable<'_>) {
    let mut state = TableState::default().with_offset(table.scroll);
    if !table.rows.is_empty() {
        state.select(Some(table.selected.min(table.rows.len() - 1)));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(table.title)
        .style(base_style(table.theme))
        .border_style(Style::default().fg(base_fg(table.theme)));
    let table = Table::new(table.rows, table.widths)
        .header(table.header)
        .row_highlight_style(Style::default().bg(selection_bg(table.theme.bg)))
        .style(base_style(table.theme))
        .block(block);
    f.render_stateful_widget(table, area, &mut state);
}

pub fn header_style(theme: &RenderTheme) -> Style {
    Style::default()
        .fg(base_fg(theme))
        .add_modifier(Modifier::BOLD)
}

pub fn centered_area(area: Rect, percent_x: u16, rows: usize, max_height: u16) -> Rect {
    let body = rows.max(1) as u16;
    let height = (body + 3).min(max_height);
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

pub fn row_at(area: Rect, rows: usize, scroll: usize, mouse_x: u16, mouse_y: u16) -> Option<usize> {
    if mouse_x < area.x || mouse_x >= area.x + area.width {
        return None;
    }
    if mouse_y < area.y || mouse_y >= area.y + area.height {
        return None;
    }
    let inner_y = mouse_y.saturating_sub(area.y + 1);
    if inner_y == 0 {
        return None;
    }
    let row = inner_y.saturating_sub(1) as usize;
    let row = row.saturating_add(scroll);
    if row < rows { Some(row) } else { None }
}

pub fn visible_rows(area: Rect) -> usize {
    area.height.saturating_sub(3).max(1) as usize
}
