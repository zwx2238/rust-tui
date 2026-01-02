use crate::render::RenderTheme;
use crate::types::ROLE_USER;
use crate::ui::draw::{
    draw_categories, draw_footer, draw_header, draw_tabs, inner_area, layout_chunks,
};
use crate::ui::notice::draw_notice;
use crate::ui::overlay_table::{OverlayTable, draw_overlay_table, header_style};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_layout::compute_sidebar_width;
use crate::ui::text_utils::truncate_to_width;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row};
use std::io::Stdout;
use std::time::Instant;

pub struct SummaryRow {
    pub tab_index: usize,
    pub tab_id: usize,
    pub category: String,
    pub message_count: usize,
    pub status: &'static str,
    pub exec_pending: bool,
    pub exec_since: Option<Instant>,
    pub latest_user: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SummarySort {
    TabOrder,
    ExecTime,
}

pub fn build_summary_rows(tabs: &[TabState], max_latest_width: usize) -> Vec<SummaryRow> {
    tabs.iter()
        .enumerate()
        .map(|(idx, tab)| {
            let status = if tab.app.busy { "generating" } else { "done" };
            let exec_pending =
                tab.app.pending_code_exec.is_some() || tab.app.code_exec_live.is_some();
            let exec_since = tab.app.pending_code_exec.as_ref().map(|p| p.requested_at);
            let latest_user = latest_user_question(&tab.app.messages)
                .map(|s| truncate_to_width(s, max_latest_width))
                .unwrap_or_else(|| "-".to_string());
            SummaryRow {
                tab_index: idx,
                tab_id: idx + 1,
                category: tab.category.clone(),
                message_count: tab.app.messages.len(),
                status,
                exec_pending,
                exec_since,
                latest_user,
            }
        })
        .collect()
}

pub fn sort_summary_rows(rows: &mut [SummaryRow], sort: SummarySort) {
    match sort {
        SummarySort::TabOrder => rows.sort_by_key(|r| r.tab_index),
        SummarySort::ExecTime => rows.sort_by_key(|r| {
            let pending_rank = if r.exec_pending { 0 } else { 1 };
            let since = r
                .exec_since
                .map(|t| t.elapsed().as_millis() as u64)
                .unwrap_or(u64::MAX);
            (pending_rank, since, r.tab_index as u64)
        }),
    }
}

pub fn redraw_summary(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tabs: &mut [TabState],
    active_tab: usize,
    tab_labels: &[String],
    active_tab_pos: usize,
    categories: &[String],
    active_category: usize,
    theme: &RenderTheme,
    startup_text: Option<&str>,
    header_note: Option<&str>,
    selected_row: usize,
    scroll: usize,
    sort: SummarySort,
) -> Result<Vec<SummaryRow>, Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let sidebar_width = compute_sidebar_width(categories, size.width);
    let (header_area, category_area, tabs_area, body_area, _input_area, footer_area) =
        layout_chunks(size, 0, sidebar_width);
    let max_latest_width = inner_area(body_area, 1, 1).width.saturating_sub(40) as usize;
    let mut rows = build_summary_rows(tabs, max_latest_width.max(10));
    sort_summary_rows(&mut rows, sort);
    terminal.draw(|f| {
        draw_header(f, header_area, theme, header_note);
        draw_categories(f, category_area, categories, active_category, theme);
        draw_tabs(
            f,
            tabs_area,
            tab_labels,
            active_tab_pos,
            theme,
            startup_text,
        );
        draw_summary_table(f, body_area, &rows, selected_row, scroll, theme, sort);
        draw_footer(f, footer_area, theme, false);
        if let Some(tab) = tabs.get_mut(active_tab) {
            draw_notice(f, size, &mut tab.app, theme);
        }
    })?;
    Ok(rows)
}

fn draw_summary_table(
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    rows: &[SummaryRow],
    selected_row: usize,
    scroll: usize,
    theme: &RenderTheme,
    sort: SummarySort,
) {
    let header = Row::new(vec![
        Cell::from("对话"),
        Cell::from("分类"),
        Cell::from("消息数"),
        Cell::from("状态"),
        Cell::from("执行中"),
        Cell::from("最新提问"),
    ])
    .style(header_style(theme));

    let body = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.tab_id.to_string()),
            Cell::from(row.category.clone()),
            Cell::from(row.message_count.to_string()),
            Cell::from(row.status),
            Cell::from(if row.exec_pending { "是" } else { "否" }),
            Cell::from(row.latest_user.clone()),
        ])
    });

    let popup = OverlayTable {
        title: Line::from(match sort {
            SummarySort::TabOrder => "汇总页 · F1 退出 · Enter 进入 · S 排序(默认)",
            SummarySort::ExecTime => "汇总页 · F1 退出 · Enter 进入 · S 排序(执行中)",
        }),
        header,
        rows: body.collect(),
        widths: vec![
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
        selected: selected_row,
        scroll,
        theme,
    };
    draw_overlay_table(f, area, popup);
}

fn latest_user_question(messages: &[crate::types::Message]) -> Option<&str> {
    messages
        .iter()
        .rev()
        .find(|m| m.role == ROLE_USER)
        .map(|m| m.content.as_str())
}

// text utilities are centralized in text_utils
