use crate::ui::logic::point_in_rect;
use crate::ui::runtime_helpers::{TabState, tab_index_at, tab_labels_for_category, visible_tab_indices};
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
            *active_tab = idx;
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
    let category = categories
        .get(active_category)
        .map(|s| s.as_str())
        .unwrap_or("默认");
    let labels = tab_labels_for_category(tabs, category);
    if let Some(pos) = tab_index_at(mouse_x, tabs_area, &labels) {
        let visible = visible_tab_indices(tabs, category);
        if let Some(idx) = visible.get(pos) {
            *active_tab = *idx;
        }
    }
    true
}
