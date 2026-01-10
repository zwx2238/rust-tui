#[cfg(test)]
mod tests {
    use crate::ui::tab_bar::{TabBarItemKind, build_tab_bar_view, hit_test_tab_bar};
    use ratatui::layout::Rect;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn build_tab_bar_view_middle_shows_both_indicators() {
        let labels = sample_labels(10);
        let view = build_tab_bar_view(&labels, 5, 20);
        assert!(
            view.items
                .iter()
                .any(|i| matches!(i.kind, TabBarItemKind::Tab(5)))
        );
        assert!(view.items.iter().any(|i| i.active));
        assert!(
            view.items
                .iter()
                .any(|i| matches!(i.kind, TabBarItemKind::MoreLeft { .. }))
        );
        assert!(
            view.items
                .iter()
                .any(|i| matches!(i.kind, TabBarItemKind::MoreRight { .. }))
        );
        assert!(view.items.iter().any(|i| matches!(i.kind, TabBarItemKind::Add)));
        assert!(view_width(&view) <= 20);
    }

    #[test]
    fn build_tab_bar_view_first_has_no_left_indicator() {
        let labels = sample_labels(10);
        let view = build_tab_bar_view(&labels, 0, 20);
        assert!(
            !view
                .items
                .iter()
                .any(|i| matches!(i.kind, TabBarItemKind::MoreLeft { .. }))
        );
        assert!(
            view.items
                .iter()
                .any(|i| matches!(i.kind, TabBarItemKind::MoreRight { .. }))
        );
        assert!(view_width(&view) <= 20);
    }

    #[test]
    fn build_tab_bar_view_last_has_no_right_indicator() {
        let labels = sample_labels(10);
        let view = build_tab_bar_view(&labels, 9, 20);
        assert!(
            view.items
                .iter()
                .any(|i| matches!(i.kind, TabBarItemKind::MoreLeft { .. }))
        );
        assert!(
            !view
                .items
                .iter()
                .any(|i| matches!(i.kind, TabBarItemKind::MoreRight { .. }))
        );
        assert!(view_width(&view) <= 20);
    }

    #[test]
    fn build_tab_bar_view_extremely_narrow_shows_add_only() {
        let labels = sample_labels(10);
        let view = build_tab_bar_view(&labels, 7, 1);
        assert_eq!(view.items.len(), 1);
        assert!(matches!(view.items[0].kind, TabBarItemKind::Add));
    }

    #[test]
    fn build_tab_bar_view_width_zero_is_empty() {
        let labels = sample_labels(10);
        let view = build_tab_bar_view(&labels, 3, 0);
        assert!(view.items.is_empty());
    }

    #[test]
    fn hit_test_tab_bar_matches_items_and_respects_separators() {
        let labels = sample_labels(10);
        let view = build_tab_bar_view(&labels, 5, 20);
        let area = Rect::new(0, 0, 20, 1);
        let total = view_width(&view) as u16;
        assert!(total > 0);

        let first = hit_test_tab_bar(0, area, &view);
        assert!(matches!(first, Some(TabBarItemKind::MoreLeft { .. })));

        let last = hit_test_tab_bar(total - 1, area, &view);
        assert!(matches!(last, Some(TabBarItemKind::Add)));

        if view.items.len() >= 2 {
            let first_w = view.items[0].label.width() as u16;
            let on_sep = hit_test_tab_bar(first_w, area, &view);
            assert_eq!(on_sep, None);
        }
    }

    fn sample_labels(n: usize) -> Vec<String> {
        (0..n).map(|i| format!(" 对话 {} ", i + 1)).collect()
    }

    fn view_width(view: &crate::ui::tab_bar::TabBarView) -> usize {
        let mut total = 0usize;
        for (i, item) in view.items.iter().enumerate() {
            if i > 0 {
                total = total.saturating_add(1);
            }
            total = total.saturating_add(item.label.width());
        }
        total
    }
}
