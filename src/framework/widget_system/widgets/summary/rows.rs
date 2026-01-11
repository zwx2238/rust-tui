use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::interaction::text_utils::truncate_to_width;

use super::SummaryRow;

pub(crate) fn build_summary_row(idx: usize, tab: &TabState, max_latest_width: usize) -> SummaryRow {
    let status = if tab.app.busy { "generating" } else { "done" };
    let exec_pending = tab.app.pending_code_exec.is_some() || tab.app.code_exec_live.is_some();
    let exec_since = tab.app.pending_code_exec.as_ref().map(|p| p.requested_at);
    let latest_user = latest_user_question(&tab.app.messages, &tab.app.default_role)
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

pub(crate) fn exec_time_sort_key(row: &SummaryRow) -> (u8, u64, u64) {
    let pending_rank = if row.exec_pending { 0 } else { 1 };
    let since = row
        .exec_since
        .map(|t| t.elapsed().as_millis() as u64)
        .unwrap_or(u64::MAX);
    (pending_rank, since, row.tab_index as u64)
}

fn latest_user_question<'a>(
    messages: &'a [crate::types::Message],
    role: &str,
) -> Option<&'a str> {
    messages
        .iter()
        .rev()
        .find(|m| m.role == role || m.role == crate::types::ROLE_USER)
        .map(|m| m.content.as_str())
}
