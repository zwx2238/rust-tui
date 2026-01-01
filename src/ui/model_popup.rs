use crate::model_registry::ModelProfile;
use crate::render::RenderTheme;
use crate::ui::popup_layout::popup_area;
use crate::ui::popup_table::{draw_table_popup, header_style, popup_row_at, popup_visible_rows, TablePopup};
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

pub fn draw_model_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    models: &[ModelProfile],
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let popup = model_popup_area(area, models.len());
    let header = Row::new(vec![
        Cell::from("名称"),
        Cell::from("模型"),
        Cell::from("Base URL"),
    ])
    .style(header_style(theme));
    let body = models.iter().map(|m| {
        Row::new(vec![
            Cell::from(m.key.clone()),
            Cell::from(m.model.clone()),
            Cell::from(m.base_url.clone()),
        ])
    });
    let popup_spec = TablePopup {
        title: Line::from("模型切换 · Enter 确认 · Esc 取消 · F3 快速切换"),
        header,
        rows: body.collect(),
        widths: vec![
            Constraint::Length(12),
            Constraint::Length(22),
            Constraint::Min(10),
        ],
        selected,
        scroll,
        theme,
    };
    draw_table_popup(f, popup, popup_spec);
}

pub fn model_popup_area(area: Rect, rows: usize) -> Rect {
    popup_area(area, 70, rows, 16)
}

pub fn model_row_at(
    area: Rect,
    rows: usize,
    scroll: usize,
    mouse_x: u16,
    mouse_y: u16,
) -> Option<usize> {
    let popup = model_popup_area(area, rows);
    popup_row_at(popup, rows, scroll, mouse_x, mouse_y)
}

pub fn model_visible_rows(area: Rect, rows: usize) -> usize {
    let popup = model_popup_area(area, rows);
    popup_visible_rows(popup)
}

// layout helpers are centralized in popup_layout

// selection color handled by popup_table
