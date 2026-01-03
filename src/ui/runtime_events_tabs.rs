use crate::ui::logic::point_in_rect;
use crate::ui::runtime_helpers::{
    TabState, tab_index_at, tab_labels_for_category, visible_tab_indices,
};
use ratatui::layout::Rect;

pub(crate) fn handle_tab_category_click(
    mouse_x: u16,
    mouse_y: u16,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &[String],
    active_category: &mut usize,
    tabs_area: Rect,
    category_area: Rect,
) -> bool {
    if handle_category_click(mouse_x, mouse_y, tabs, active_tab, categories, active_category, category_area) {
        return true;
    }
    handle_tabs_click(mouse_x, mouse_y, tabs, active_tab, categories, *active_category, tabs_area)
}

fn handle_category_click(
    mouse_x: u16,
    mouse_y: u16,
    tabs: &mut Vec<TabState>,
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
        if let Some(category) = categories.get(*active_category) {
            let visible = visible_tab_indices(tabs, category);
            if let Some(tab_idx) = visible.first() {
                *active_tab = *tab_idx;
            }
        }
    }
    true
}

fn handle_tabs_click(
    mouse_x: u16,
    mouse_y: u16,
    tabs: &mut Vec<TabState>,
    active_tab: &mut usize,
    categories: &[String],
    active_category: usize,
    tabs_area: Rect,
) -> bool {
    if !point_in_rect(mouse_x, mouse_y, tabs_area) {
        return false;
    }
    let category = categories.get(active_category).map(|s| s.as_str()).unwrap_or("默认");
    let labels = tab_labels_for_category(tabs, category);
    if let Some(pos) = tab_index_at(mouse_x, tabs_area, &labels) {
        let visible = visible_tab_indices(tabs, category);
        if let Some(idx) = visible.get(pos) {
            *active_tab = *idx;
        }
    }
    true
}
