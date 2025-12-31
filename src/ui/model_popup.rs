use crate::model_registry::ModelProfile;
use crate::render::RenderTheme;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

pub fn draw_model_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    models: &[ModelProfile],
    selected: usize,
    theme: &RenderTheme,
) {
    let popup = model_popup_area(area, models.len());
    let header = Row::new(vec![
        Cell::from("名称"),
        Cell::from("模型"),
        Cell::from("Base URL"),
    ])
    .style(
        Style::default()
            .fg(theme.fg.unwrap_or(Color::White))
            .add_modifier(Modifier::BOLD),
    );
    let body = models.iter().map(|m| {
        Row::new(vec![
            Cell::from(m.key.clone()),
            Cell::from(m.model.clone()),
            Cell::from(m.base_url.clone()),
        ])
    });
    let mut state = TableState::default();
    if !models.is_empty() {
        state.select(Some(selected.min(models.len() - 1)));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title_top(Line::from("模型切换 · Enter 确认 · Esc 取消 · F3 快速切换"))
        .style(Style::default().bg(theme.bg).fg(theme.fg.unwrap_or(Color::White)))
        .border_style(Style::default().fg(theme.fg.unwrap_or(Color::White)));
    let table = Table::new(body, [
        Constraint::Length(12),
        Constraint::Length(22),
        Constraint::Min(10),
    ])
    .header(header)
    .row_highlight_style(Style::default().bg(selection_bg(theme.bg)))
    .style(Style::default().bg(theme.bg).fg(theme.fg.unwrap_or(Color::White)))
    .block(block);
    f.render_stateful_widget(table, popup, &mut state);
}

pub fn model_popup_area(area: Rect, rows: usize) -> Rect {
    centered_rect(area, 70, popup_height(rows))
}

pub fn model_row_at(area: Rect, rows: usize, mouse_x: u16, mouse_y: u16) -> Option<usize> {
    let popup = model_popup_area(area, rows);
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
    if row < rows {
        Some(row)
    } else {
        None
    }
}

fn popup_height(rows: usize) -> u16 {
    let body = rows.max(1) as u16;
    (body + 3).min(16)
}

fn centered_rect(area: Rect, percent_x: u16, height: u16) -> Rect {
    let width = area.width * percent_x / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let h = height.min(area.height.saturating_sub(2)).max(3);
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width, height: h }
}

fn selection_bg(bg: Color) -> Color {
    match bg {
        Color::White => Color::Gray,
        _ => Color::DarkGray,
    }
}
