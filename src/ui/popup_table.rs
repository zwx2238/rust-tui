use crate::render::RenderTheme;
use crate::ui::draw::style::{base_fg, base_style, selection_bg};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, Table, TableState};

pub struct TablePopup<'a> {
    pub title: Line<'a>,
    pub header: Row<'a>,
    pub rows: Vec<Row<'a>>,
    pub widths: Vec<Constraint>,
    pub selected: usize,
    pub scroll: usize,
    pub theme: &'a RenderTheme,
}

pub fn draw_table_popup(f: &mut ratatui::Frame<'_>, area: Rect, popup: TablePopup<'_>) {
    let mut state = TableState::default().with_offset(popup.scroll);
    if !popup.rows.is_empty() {
        state.select(Some(popup.selected.min(popup.rows.len() - 1)));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(popup.title)
        .style(base_style(popup.theme))
        .border_style(Style::default().fg(base_fg(popup.theme)));
    let table = Table::new(popup.rows, popup.widths)
        .header(popup.header)
        .row_highlight_style(Style::default().bg(selection_bg(popup.theme.bg)))
        .style(base_style(popup.theme))
        .block(block);
    f.render_stateful_widget(table, area, &mut state);
}

pub fn header_style(theme: &RenderTheme) -> Style {
    Style::default()
        .fg(base_fg(theme))
        .add_modifier(Modifier::BOLD)
}

pub fn popup_row_at(
    popup: Rect,
    rows: usize,
    scroll: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    if mouse_x < popup.x || mouse_x >= popup.x + popup.width {
        return None;
    }
    if mouse_y < popup.y || mouse_y >= popup.y + popup.height {
        return None;
    }
    let inner_y = mouse_y.saturating_sub(popup.y + 1);
    if inner_y == 0 {
        return None;
    }
    let row = inner_y.saturating_sub(1) as usize;
    let row = row.saturating_add(scroll);
    if row < rows {
        Some(row)
    } else {
        None
    }
}

pub fn popup_visible_rows(popup: Rect) -> usize {
    popup.height.saturating_sub(3).max(1) as usize
}

// selection_bg moved to draw::style
