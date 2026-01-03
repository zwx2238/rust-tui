use crate::render::RenderTheme;
use crate::ui::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};

use super::{SummaryRow, SummarySort};

pub(crate) fn draw_summary_table(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    rows: &[SummaryRow],
    selected_row: usize,
    scroll: usize,
    theme: &RenderTheme,
    sort: SummarySort,
) {
    let popup = build_summary_table(rows, selected_row, scroll, theme, sort);
    draw_overlay_table(f, area, popup);
}

fn build_summary_table<'a>(
    rows: &'a [SummaryRow],
    selected_row: usize,
    scroll: usize,
    theme: &'a RenderTheme,
    sort: SummarySort,
) -> OverlayTable<'a> {
    let header = summary_header(theme);
    let body = summary_body(rows);
    OverlayTable {
        title: Line::from(summary_title(sort)),
        header,
        rows: body.collect(),
        widths: summary_widths(),
        selected: selected_row,
        scroll,
        theme,
    }
}

fn summary_header(theme: &RenderTheme) -> Row<'static> {
    Row::new(vec![
        Cell::from("对话"),
        Cell::from("分类"),
        Cell::from("消息数"),
        Cell::from("状态"),
        Cell::from("执行中"),
        Cell::from("最新提问"),
    ])
    .style(header_style(theme))
}

fn summary_body<'a>(rows: &'a [SummaryRow]) -> impl Iterator<Item = Row<'a>> + 'a {
    rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.tab_id.to_string()),
            Cell::from(row.category.clone()),
            Cell::from(row.message_count.to_string()),
            Cell::from(row.status),
            Cell::from(if row.exec_pending { "是" } else { "否" }),
            Cell::from(row.latest_user.clone()),
        ])
    })
}

fn summary_widths() -> Vec<Constraint> {
    vec![
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Min(10),
    ]
}

fn summary_title(sort: SummarySort) -> &'static str {
    match sort {
        SummarySort::TabOrder => "汇总页 · F1 退出 · Enter 进入 · S 排序(默认)",
        SummarySort::ExecTime => "汇总页 · F1 退出 · Enter 进入 · S 排序(执行中)",
    }
}
