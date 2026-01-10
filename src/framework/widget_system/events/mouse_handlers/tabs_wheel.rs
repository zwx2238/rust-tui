use crate::framework::widget_system::runtime::runtime_helpers::{
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
            set_active_tab(tabs, active_tab, idx);
        }
        return;
    }
    if pos == 0 {
        return;
    }
    set_active_tab(tabs, active_tab, visible[pos - 1]);
}

fn set_active_tab(tabs: &mut [TabState], active_tab: &mut usize, idx: usize) {
    if *active_tab == idx {
        return;
    }
    *active_tab = idx;
    if let Some(tab) = tabs.get_mut(idx) {
        tab.app.follow = false;
    }
}
