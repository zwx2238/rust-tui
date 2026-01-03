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

struct SummaryLayout {
    size: Rect,
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    body_area: Rect,
    footer_area: Rect,
    max_latest_width: usize,
}

struct SummaryDrawArgs<'a> {
    theme: &'a RenderTheme,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    header_note: Option<&'a str>,
    startup_text: Option<&'a str>,
    layout: &'a SummaryLayout,
    rows: &'a [SummaryRow],
    selected_row: usize,
    scroll: usize,
    sort: SummarySort,
    active_tab: Option<&'a mut TabState>,
}

pub(crate) struct RedrawSummaryParams<'a> {
    pub terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    pub tabs: &'a mut [TabState],
    pub active_tab: usize,
    pub tab_labels: &'a [String],
    pub active_tab_pos: usize,
    pub categories: &'a [String],
    pub active_category: usize,
    pub theme: &'a RenderTheme,
    pub startup_text: Option<&'a str>,
    pub header_note: Option<&'a str>,
    pub selected_row: usize,
    pub scroll: usize,
    pub sort: SummarySort,
}

struct DrawSummaryLayoutParams<'a, 'b> {
    f: &'a mut ratatui::Frame<'b>,
    theme: &'a RenderTheme,
    tab_labels: &'a [String],
    active_tab_pos: usize,
    categories: &'a [String],
    active_category: usize,
    header_note: Option<&'a str>,
    startup_text: Option<&'a str>,
    header_area: Rect,
    category_area: Rect,
    tabs_area: Rect,
    footer_area: Rect,
}

pub fn build_summary_rows(tabs: &[TabState], max_latest_width: usize) -> Vec<SummaryRow> {
    tabs.iter()
        .enumerate()
        .map(|(idx, tab)| build_summary_row(idx, tab, max_latest_width))
        .collect()
}

fn build_summary_row(idx: usize, tab: &TabState, max_latest_width: usize) -> SummaryRow {
    let status = if tab.app.busy { "generating" } else { "done" };
    let exec_pending = tab.app.pending_code_exec.is_some() || tab.app.code_exec_live.is_some();
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
}

pub fn sort_summary_rows(rows: &mut [SummaryRow], sort: SummarySort) {
    match sort {
        SummarySort::TabOrder => rows.sort_by_key(|r| r.tab_index),
        SummarySort::ExecTime => rows.sort_by_key(exec_time_sort_key),
    }
}

fn exec_time_sort_key(row: &SummaryRow) -> (u8, u64, u64) {
    let pending_rank = if row.exec_pending { 0 } else { 1 };
    let since = row
        .exec_since
        .map(|t| t.elapsed().as_millis() as u64)
        .unwrap_or(u64::MAX);
    (pending_rank, since, row.tab_index as u64)
}

pub fn redraw_summary(
    params: RedrawSummaryParams<'_>,
) -> Result<Vec<SummaryRow>, Box<dyn std::error::Error>> {
    let layout = build_summary_layout(params.terminal.size()?, params.categories);
    let mut rows = build_summary_rows(params.tabs, layout.max_latest_width.max(10));
    sort_summary_rows(&mut rows, params.sort);
    params.terminal.draw(|f| {
        draw_summary_frame(
            f,
            SummaryDrawArgs {
                theme: params.theme,
                tab_labels: params.tab_labels,
                active_tab_pos: params.active_tab_pos,
                categories: params.categories,
                active_category: params.active_category,
                header_note: params.header_note,
                startup_text: params.startup_text,
                layout: &layout,
                rows: &rows,
                selected_row: params.selected_row,
                scroll: params.scroll,
                sort: params.sort,
                active_tab: params.tabs.get_mut(params.active_tab),
            },
        );
    })?;
    Ok(rows)
}

fn build_summary_layout(size: ratatui::layout::Size, categories: &[String]) -> SummaryLayout {
    let size = Rect::new(0, 0, size.width, size.height);
    let sidebar_width = compute_sidebar_width(categories, size.width);
    let (header_area, category_area, tabs_area, body_area, _input_area, footer_area) =
        layout_chunks(size, 0, sidebar_width);
    SummaryLayout {
        size,
        header_area,
        category_area,
        tabs_area,
        body_area,
        footer_area,
        max_latest_width: max_latest_question_width(body_area),
    }
}

fn max_latest_question_width(body_area: Rect) -> usize {
    inner_area(body_area, 1, 1).width.saturating_sub(40) as usize
}

fn draw_summary_frame(f: &mut ratatui::Frame<'_>, args: SummaryDrawArgs<'_>) {
    draw_summary_layout(DrawSummaryLayoutParams {
        f,
        theme: args.theme,
        tab_labels: args.tab_labels,
        active_tab_pos: args.active_tab_pos,
        categories: args.categories,
        active_category: args.active_category,
        header_note: args.header_note,
        startup_text: args.startup_text,
        header_area: args.layout.header_area,
        category_area: args.layout.category_area,
        tabs_area: args.layout.tabs_area,
        footer_area: args.layout.footer_area,
    });
    draw_summary_table(
        f,
        args.layout.body_area,
        args.rows,
        args.selected_row,
        args.scroll,
        args.theme,
        args.sort,
    );
    if let Some(tab) = args.active_tab {
        draw_notice(f, args.layout.size, &mut tab.app, args.theme);
    }
}

fn draw_summary_layout<'a, 'b>(params: DrawSummaryLayoutParams<'a, 'b>) {
    draw_header(
        params.f,
        params.header_area,
        params.theme,
        params.header_note,
    );
    draw_categories(
        params.f,
        params.category_area,
        params.categories,
        params.active_category,
        params.theme,
    );
    draw_tabs(
        params.f,
        params.tabs_area,
        params.tab_labels,
        params.active_tab_pos,
        params.theme,
        params.startup_text,
    );
    draw_footer(params.f, params.footer_area, params.theme, false);
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

fn latest_user_question(messages: &[crate::types::Message]) -> Option<&str> {
    messages
        .iter()
        .rev()
        .find(|m| m.role == ROLE_USER)
        .map(|m| m.content.as_str())
}

// text utilities are centralized in text_utils
