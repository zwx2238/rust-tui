use ratatui::layout::Rect;

pub(crate) fn compute_terminal_popup_layout(
    area: Rect,
) -> crate::ui::terminal_popup_layout::TerminalPopupLayout {
    crate::ui::terminal_popup_layout::terminal_popup_layout(area)
}
