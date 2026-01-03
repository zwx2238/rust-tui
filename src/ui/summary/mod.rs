mod layout;
mod rows;
mod table;

use crate::render::RenderTheme;
use crate::ui::notice::draw_notice;
use crate::ui::runtime_helpers::TabState;
use layout::{SummaryLayout, build_summary_layout, draw_summary_layout};
use table::draw_summary_table;

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
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

pub fn build_summary_rows(tabs: &[TabState], max_latest_width: usize) -> Vec<SummaryRow> {
    tabs.iter()
        .enumerate()
        .map(|(idx, tab)| rows::build_summary_row(idx, tab, max_latest_width))
        .collect()
}

pub fn sort_summary_rows(rows: &mut [SummaryRow], sort: SummarySort) {
    match sort {
        SummarySort::TabOrder => rows.sort_by_key(|r| r.tab_index),
        SummarySort::ExecTime => rows.sort_by_key(rows::exec_time_sort_key),
    }
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

fn draw_summary_frame(f: &mut ratatui::Frame<'_>, args: SummaryDrawArgs<'_>) {
    draw_summary_layout(layout::DrawSummaryLayoutParams {
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
