#[cfg(test)]
mod tests {
    use crate::ui::file_patch_popup_layout::{OUTER_MARGIN, file_patch_popup_layout};
    use ratatui::layout::Rect;

    #[test]
    fn layout_positions_buttons() {
        let area = Rect::new(0, 0, 100, 30);
        let layout = file_patch_popup_layout(area);
        assert!(layout.popup.width > 0);
        assert!(layout.apply_btn.x >= OUTER_MARGIN);
        assert!(layout.cancel_btn.width > 0);
    }
}
