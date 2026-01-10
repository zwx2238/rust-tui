use crate::ui::logic::point_in_rect;
use crate::ui::runtime_helpers::{
    TabState, active_tab_position, tab_labels_for_category, visible_tab_indices,
};
use crate::ui::state::PendingCommand;
use crate::ui::tab_bar::{TabBarItemKind, build_tab_bar_view, hit_test_tab_bar};
use ratatui::layout::Rect;

pub(crate) struct TabCategoryClickParams<'a> {
    pub mouse_x: u16,
    pub mouse_y: u16,
    pub tabs: &'a mut [TabState],
    pub active_tab: &'a mut usize,
    pub categories: &'a [String],
    pub active_category: &'a mut usize,
    pub tabs_area: Rect,
    pub category_area: Rect,
}

pub(crate) fn handle_tab_category_click(params: TabCategoryClickParams<'_>) -> bool {
    if handle_category_click(
        params.mouse_x,
        params.mouse_y,
        params.tabs,
        params.active_tab,
        params.categories,
        params.active_category,
        params.category_area,
    ) {
        return true;
    }
    handle_tabs_click(
        params.mouse_x,
        params.mouse_y,
        params.tabs,
        params.active_tab,
        params.categories,
        *params.active_category,
        params.tabs_area,
    )
}

fn handle_category_click(
    mouse_x: u16,
    mouse_y: u16,
    tabs: &mut [TabState],
    active_tab: &mut usize,
    categories: &[String],
    active_category: &mut usize,
    category_area: Rect,
) -> bool {
    if !point_in_rect(mouse_x, mouse_y, category_area) {
        return false;
    }
    let row = mouse_y.saturating_sub(category_area.y) as usize;
    if row < categories.len() {
        *active_category = row;
        if let Some(category) = categories.get(row)
            && let Some(idx) = tabs.iter().position(|t| t.category == category.as_str())
        {
            set_active_tab(tabs, active_tab, idx);
        }
    }
    true
}

fn handle_tabs_click(
    mouse_x: u16,
    mouse_y: u16,
    tabs: &mut [TabState],
    active_tab: &mut usize,
    categories: &[String],
    active_category: usize,
    tabs_area: Rect,
) -> bool {
    if !point_in_rect(mouse_x, mouse_y, tabs_area) {
        return false;
    }
    let category = active_category_name(categories, active_category);
    let labels = tab_labels_for_category(tabs, category);
    let visible = visible_tab_indices(tabs, category);
    let active_pos = active_tab_position(tabs, category, *active_tab);
    let view = build_tab_bar_view(&labels, active_pos, tabs_area.width);
    hit_test_tab_bar(mouse_x, tabs_area, &view)
        .map(|kind| handle_tab_bar_hit(kind, tabs, active_tab, &visible))
        .unwrap_or(true)
}

fn active_category_name(categories: &[String], active_category: usize) -> &str {
    categories
        .get(active_category)
        .map(|s| s.as_str())
        .unwrap_or("默认")
}

fn handle_tab_bar_hit(
    kind: TabBarItemKind,
    tabs: &mut [TabState],
    active_tab: &mut usize,
    visible: &[usize],
) -> bool {
    if matches!(kind, TabBarItemKind::Add) {
        if let Some(tab) = tabs.get_mut(*active_tab) {
            tab.app.pending_command = Some(PendingCommand::NewTab);
        }
        return true;
    }
    let target = match kind {
        TabBarItemKind::Tab(pos) => visible.get(pos).copied(),
        TabBarItemKind::MoreLeft { target_pos } => visible.get(target_pos).copied(),
        TabBarItemKind::MoreRight { target_pos } => visible.get(target_pos).copied(),
        TabBarItemKind::Add => None,
    };
    if let Some(idx) = target {
        set_active_tab(tabs, active_tab, idx);
    }
    true
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
