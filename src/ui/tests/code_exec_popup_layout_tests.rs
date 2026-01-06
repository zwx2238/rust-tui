#[cfg(test)]
mod tests {
    use crate::ui::code_exec_popup_layout::{OUTER_MARGIN, code_exec_popup_layout};
    use ratatui::layout::Rect;

    #[test]
    fn layout_with_reason_has_reason_area() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = code_exec_popup_layout(area, true);
        assert!(layout.reason_input_area.width > 0);
        assert!(layout.popup.width > 0);
        assert!(layout.popup.x >= OUTER_MARGIN);
    }

    #[test]
    fn layout_without_reason_hides_reason_area() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = code_exec_popup_layout(area, false);
        assert_eq!(layout.reason_input_area.width, 0);
        assert_eq!(layout.reason_input_area.height, 0);
    }
}
