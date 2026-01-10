pub(crate) mod layout;
mod rows;
pub(crate) mod table;

use crate::ui::runtime_helpers::TabState;
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
        .map(|(idx, tab)| rows::build_summary_row(idx, tab, max_latest_width))
        .collect()
}

pub fn sort_summary_rows(rows: &mut [SummaryRow], sort: SummarySort) {
    match sort {
        SummarySort::TabOrder => rows.sort_by_key(|r| r.tab_index),
        SummarySort::ExecTime => rows.sort_by_key(rows::exec_time_sort_key),
    }
}
// 旧的 Summary「直接重绘」入口已被 framework/widget_system 替代。
