use crate::model_registry::ModelProfile;
use crate::render::RenderTheme;
use crate::ui::overlay_table::{OverlayTable, centered_area, draw_overlay_table, header_style};
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

const POPUP_MAX_HEIGHT: u16 = 16;

pub fn draw_model_popup(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    models: &[ModelProfile],
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let popup = model_popup_area(area, models.len());
    let popup_spec = build_model_table(models, selected, scroll, theme);
    draw_overlay_table(f, popup, popup_spec);
}

fn build_model_table<'a>(
    models: &'a [ModelProfile],
    selected: usize,
    scroll: usize,
    theme: &'a RenderTheme,
) -> OverlayTable<'a> {
    OverlayTable {
        title: Line::from(model_title()),
        header: model_header(theme),
        rows: model_body(models),
        widths: model_widths(),
        selected,
        scroll,
        theme,
    }
}

fn model_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("名称"),
        Cell::from("模型"),
        Cell::from("Base URL"),
    ])
    .style(header_style(theme))
}

fn model_body<'a>(models: &'a [ModelProfile]) -> Vec<Row<'a>> {
    models
        .iter()
        .map(|m| {
            Row::new(vec![
                Cell::from(m.key.clone()),
                Cell::from(m.model.clone()),
                Cell::from(m.base_url.clone()),
            ])
        })
        .collect()
}

fn model_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(12),
        Constraint::Length(22),
        Constraint::Min(10),
    ]
}

fn model_title() -> &'static str {
    "模型切换 · Enter 确认 · Esc 取消 · F3/Shift+F3 快速切换"
}

pub fn model_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 70, rows, POPUP_MAX_HEIGHT)
}

// layout helpers are centralized in overlay_table

// selection color handled by overlay_table
