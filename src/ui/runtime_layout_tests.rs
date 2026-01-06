#[cfg(test)]
mod tests {
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_layout::{compute_layout, compute_sidebar_width};
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
        let layout1 = compute_layout(
            Rect::new(0, 0, 80, 24),
            &view,
            &tabs1,
            0,
            &["cat".to_string()],
        );
        let mut tabs3 = tabs1;
        tabs3[0].app.input.insert_str("\nline2\nline3");
        let layout3 = compute_layout(
            Rect::new(0, 0, 80, 24),
            &view,
            &tabs3,
            0,
            &["cat".to_string()],
        );
        assert!(layout3.input_area.height >= layout1.input_area.height);
        assert!(layout3.msg_width > 0);
    }
}
