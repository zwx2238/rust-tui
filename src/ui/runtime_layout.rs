use crate::ui::draw::{inner_height, inner_width, layout_chunks};
use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_view::ViewState;
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthStr;

pub(crate) struct LayoutInfo {
    pub(crate) header_area: Rect,
    pub(crate) category_area: Rect,
    pub(crate) tabs_area: Rect,
    pub(crate) msg_area: Rect,
    pub(crate) input_area: Rect,
    pub(crate) footer_area: Rect,
    pub(crate) msg_width: usize,
    pub(crate) view_height: u16,
    pub(crate) input_height: u16,
}

pub(crate) fn compute_layout(
    size: Rect,
    view: &ViewState,
    tabs: &[TabState],
    active_tab: usize,
    categories: &[String],
) -> LayoutInfo {
    let input_height = compute_input_height(size, view, tabs, active_tab);
    let sidebar_width = compute_sidebar_width(categories, size.width);
    let (header_area, category_area, tabs_area, msg_area, input_area, footer_area) =
        layout_chunks(size, input_height, sidebar_width);
    let msg_width = inner_width(msg_area, 1);
    let view_height = inner_height(msg_area, 0);
    LayoutInfo {
        header_area,
        category_area,
        tabs_area,
        msg_area,
        input_area,
        footer_area,
        msg_width,
        view_height,
        input_height,
    }
}

pub(crate) fn compute_sidebar_width(categories: &[String], total_width: u16) -> u16 {
    let max_label = categories.iter().map(|c| c.width()).max().unwrap_or(4);
    let desired = (max_label as u16).saturating_add(2).clamp(8, 20);
    let max_allowed = total_width.saturating_sub(20).max(8);
    desired.min(max_allowed)
}

fn compute_input_height(size: Rect, view: &ViewState, tabs: &[TabState], active_tab: usize) -> u16 {
    if view.overlay.uses_simple_layout() {
        return 0;
    }
    let input_lines = tabs
        .get(active_tab)
        .map(|tab| tab.app.input.lines().len())
        .unwrap_or(1)
        .max(1);
    let min_inner_lines = 5usize;
    let max_inner_lines = 10usize;
    let max_input_height = size.height.saturating_sub(1).saturating_sub(3).max(1);
    let max_inner_lines_available = max_input_height.saturating_sub(2) as usize;
    let inner_lines = input_lines
        .clamp(min_inner_lines, max_inner_lines)
        .min(max_inner_lines_available.max(1));
    (inner_lines as u16).saturating_add(2)
}
