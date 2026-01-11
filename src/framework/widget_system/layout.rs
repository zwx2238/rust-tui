use crate::framework::widget_system::runtime::runtime_helpers::TabState;
use crate::framework::widget_system::runtime::runtime_view::ViewState;
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
}

pub(crate) fn empty_layout_info() -> LayoutInfo {
    let z = Rect::new(0, 0, 0, 0);
    LayoutInfo {
        header_area: z,
        category_area: z,
        tabs_area: z,
        msg_area: z,
        input_area: z,
        footer_area: z,
        msg_width: 0,
        view_height: 0,
    }
}

pub(crate) fn compute_sidebar_width(categories: &[String], total_width: u16) -> u16 {
    let max_label = categories.iter().map(|c| c.width()).max().unwrap_or(4);
    let desired = (max_label as u16).saturating_add(2).clamp(8, 20);
    let max_allowed = total_width.saturating_sub(20).max(8);
    desired.min(max_allowed)
}

pub(crate) fn compute_history_width(total_width: u16) -> u16 {
    let min_msg_width = 20u16;
    let collapse_below = 40u16;
    if total_width < collapse_below {
        return 0;
    }
    let desired = 24u16;
    let max_allowed = total_width.saturating_sub(min_msg_width);
    desired.min(max_allowed)
}

pub(crate) fn compute_input_height(
    size: Rect,
    view: &ViewState,
    tabs: &[TabState],
    active_tab: usize,
) -> u16 {
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
