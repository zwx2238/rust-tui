use super::popup_layout::{TerminalPopupLayout, terminal_popup_layout};
use ratatui::layout::Rect;

pub(crate) fn compute_terminal_popup_layout(
    area: Rect,
) -> TerminalPopupLayout {
    terminal_popup_layout(area)
}
