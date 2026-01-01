use crate::ui::draw::{inner_height, inner_width, layout_chunks};
use crate::ui::runtime_helpers::TabState;
use crate::ui::overlay::OverlayKind;
use crate::ui::runtime_view::ViewState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(crate) struct LayoutInfo {
    pub(crate) tabs_area: Rect,
    pub(crate) msg_area: Rect,
    pub(crate) input_area: Rect,
    pub(crate) msg_width: usize,
    pub(crate) view_height: u16,
    pub(crate) input_height: u16,
}

pub(crate) fn compute_layout(
    size: Rect,
    view: &ViewState,
    tabs: &[TabState],
    active_tab: usize,
) -> LayoutInfo {
    if matches!(
        view.overlay.active,
        Some(OverlayKind::Summary | OverlayKind::Jump)
    ) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(3)].as_ref())
            .split(size);
        let tabs_area = layout[0];
        let msg_area = layout[1];
        let msg_width = inner_width(msg_area, 1);
        let view_height = inner_height(msg_area, 0) as u16;
        LayoutInfo {
            tabs_area,
            msg_area,
            input_area: Rect::default(),
            msg_width,
            view_height,
            input_height: 0,
        }
    } else {
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
        let input_height = (inner_lines as u16).saturating_add(2);
        let (tabs_area, msg_area, input_area) = layout_chunks(size, input_height);
        let msg_width = inner_width(msg_area, 1);
        let view_height = inner_height(msg_area, 0) as u16;
        LayoutInfo {
            tabs_area,
            msg_area,
            input_area,
            msg_width,
            view_height,
            input_height,
        }
    }
}
