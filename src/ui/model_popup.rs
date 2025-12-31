use crate::model_registry::ModelProfile;
use crate::render::RenderTheme;
use crate::ui::popup_table::{draw_table_popup, popup_row_at, TablePopup};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
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
    centered_rect(area, 70, popup_height(rows))
}

pub fn model_row_at(area: Rect, rows: usize, mouse_x: u16, mouse_y: u16) -> Option<usize> {
    let popup = model_popup_area(area, rows);
    popup_row_at(popup, rows, 0, mouse_x, mouse_y)
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

// selection color handled by popup_table
