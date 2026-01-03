use crate::render::RenderTheme;
use crate::ui::commands::all_commands;
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
    let rows = help_rows();
    let popup = help_popup_area(area, rows.len());
    let table = build_help_table(&rows, selected, scroll, theme);
    draw_overlay_table(f, popup, table);
}

fn build_help_table<'a>(
    rows: &'a [HelpRow],
    selected: usize,
    scroll: usize,
    theme: &'a RenderTheme,
) -> OverlayTable<'a> {
    let header = help_table_header(theme);
    let body = help_table_rows(rows);
    OverlayTable {
        title: Line::from("帮助 · F10/Esc 退出"),
        header,
        rows: body,
        widths: help_table_widths(),
        selected,
        scroll,
        theme,
    }
}

fn help_table_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("类型"),
        Cell::from("触发"),
        Cell::from("说明"),
    ])
    .style(header_style(theme))
}

fn help_table_rows(rows: &[HelpRow]) -> Vec<Row<'static>> {
    rows.iter()
        .map(|row| {
            Row::new(vec![
                Cell::from(row.kind),
                Cell::from(row.trigger.clone()),
                Cell::from(row.description),
            ])
        })
        .collect()
}

fn help_table_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(8),
        Constraint::Length(20),
        Constraint::Min(10),
    ]
}

pub(crate) fn help_rows_len() -> usize {
    help_rows().len()
}

pub(crate) fn help_popup_area(area: Rect, rows: usize) -> Rect {
    centered_area(area, 90, rows, POPUP_MAX_HEIGHT)
}

struct HelpRow {
    kind: &'static str,
    trigger: String,
    description: &'static str,
}

fn help_rows() -> Vec<HelpRow> {
    let mut rows = Vec::new();
    for s in all_shortcuts() {
        rows.push(HelpRow {
            kind: "快捷键",
            trigger: s.keys.to_string(),
            description: s.description,
        });
    }
    for c in all_commands() {
        let trigger = if c.args.is_empty() {
            c.name.to_string()
        } else {
            format!("{} {}", c.name, c.args)
        };
        rows.push(HelpRow {
            kind: "命令",
            trigger,
            description: c.description,
        });
    }
    rows
}
