use crate::render::RenderTheme;
use crate::ui::overlay_table::{OverlayTable, centered_area, draw_overlay_table, header_style};
use crate::ui::shortcuts::all_shortcuts;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

const POPUP_MAX_HEIGHT: u16 = 24;

pub(crate) fn draw_shortcut_help(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    selected: usize,
    scroll: usize,
    theme: &RenderTheme,
) {
    let shortcuts = all_shortcuts();
    let popup = help_popup_area(area, shortcuts.len());
    let header = Row::new(vec![
        Cell::from("区域"),
        Cell::from("按键"),
        Cell::from("说明"),
    ])
    .style(header_style(theme));
    let body = shortcuts.iter().map(|s| {
        Row::new(vec![
            Cell::from(s.scope.label()),
            Cell::from(s.keys),
            Cell::from(s.description),
        ])
    });
    let popup_spec = OverlayTable {
        title: Line::from("快捷键 · F10/Esc 退出"),
        header,
        rows: body.collect(),
        widths: vec![Constraint::Length(6), Constraint::Length(16), Constraint::Min(10)],
        selected,
        scroll,
        theme,
    };
    draw_overlay_table(f, popup, popup_spec);
}

pub(crate) fn help_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 90, rows, POPUP_MAX_HEIGHT)
}
