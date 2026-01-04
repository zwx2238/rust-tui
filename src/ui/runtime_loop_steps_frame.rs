use crate::ui::runtime_helpers::TabState;
use crate::ui::runtime_layout::{LayoutInfo, compute_layout};
use crate::ui::runtime_view::ViewState;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};

pub(crate) struct FrameLayout {
    pub(crate) size: Rect,
    pub(crate) layout: LayoutInfo,
}

pub(crate) fn frame_layout(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    view: &ViewState,
    tabs: &[TabState],
    active_tab: usize,
    categories: &[String],
) -> Result<FrameLayout, Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let size = Rect::new(0, 0, size.width, size.height);
    let layout = compute_layout(size, view, tabs, active_tab, categories);
    Ok(FrameLayout { size, layout })
}

pub(crate) fn prepare_categories(
    tabs: &[TabState],
    active_tab: usize,
    categories: &mut Vec<String>,
    active_category: &mut usize,
) -> String {
    ensure_categories(categories);
    clamp_active_category(categories, active_category);
    categories[*active_category].clone()
}

pub(crate) fn tab_labels_and_pos(
    tabs: &[TabState],
    active_tab: usize,
    active_category_name: &str,
) -> (Vec<String>, usize) {
    let tab_labels =
        crate::ui::runtime_helpers::tab_labels_for_category(tabs, active_category_name);
    let active_tab_pos =
        crate::ui::runtime_helpers::active_tab_position(tabs, active_category_name, active_tab);
    (tab_labels, active_tab_pos)
}

fn ensure_categories(categories: &mut Vec<String>) {
    if categories.is_empty() {
        categories.push("默认".to_string());
    }
}

fn clamp_active_category(categories: &[String], active_category: &mut usize) {
    if *active_category >= categories.len() {
        *active_category = 0;
    }
}

// active_category is user-driven; do not auto-sync it to active_tab here.
