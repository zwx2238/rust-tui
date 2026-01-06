#[cfg(test)]
mod tests {
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_layout::{compute_input_height, compute_sidebar_width};
    use crate::ui::runtime_view::ViewState;
    use ratatui::layout::Rect;

    #[test]
    fn sidebar_width_clamped() {
        let categories = vec!["短".to_string(), "很长的分类名称".to_string()];
        let width = compute_sidebar_width(&categories, 80);
        assert!(width >= 8);
        assert!(width <= 20);
    }

    #[test]
    fn compute_layout_uses_input_lines() {
        let view = ViewState::new();
        let mut tabs1 = vec![TabState::new(
            "id".into(),
            "cat".into(),
            "",
            false,
            "m",
            "p",
        )];
        tabs1[0].app.input.insert_str("line1");
        let size = Rect::new(0, 0, 80, 24);
        let input_height1 = compute_input_height(size, &view, &tabs1, 0);
        let mut tabs3 = tabs1;
        tabs3[0].app.input.insert_str("\nline2\nline3");
        let input_height3 = compute_input_height(size, &view, &tabs3, 0);
        assert!(input_height3 >= input_height1);
        assert!(compute_sidebar_width(&["cat".to_string()], size.width) >= 8);
    }
}
