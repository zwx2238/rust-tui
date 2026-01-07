use crate::ui::runtime_helpers::{
    TabState, active_tab_position, tab_position_in_category, visible_tab_indices,
};

pub(crate) fn handle_tabs_wheel(
    tabs: &mut [TabState],
    active_tab: &mut usize,
    categories: &[String],
    active_category: usize,
    down: bool,
) {
    let category = categories
        .get(active_category)
        .map(|s| s.as_str())
        .unwrap_or("默认");
    let visible = visible_tab_indices(tabs, category);
    if visible.len() <= 1 {
        return;
    }
    let pos = tab_position_in_category(tabs, category, *active_tab)
        .unwrap_or_else(|| active_tab_position(tabs, category, *active_tab));
    let pos = pos.min(visible.len().saturating_sub(1));

    if down {
        if let Some(idx) = visible.get(pos + 1).copied() {
            *active_tab = idx;
        }
        return;
    }
    if pos == 0 {
        return;
    }
    *active_tab = visible[pos - 1];
}
