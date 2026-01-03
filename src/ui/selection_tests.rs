#[cfg(test)]
mod tests {
    use crate::ui::selection::{
        Selection, apply_selection_to_text, chat_position_from_mouse, extract_selection,
        line_width, slice_line_by_cols,
    };
    use ratatui::layout::Rect;
    use ratatui::style::Style;
    use ratatui::text::{Line, Span, Text};

    #[test]
    fn selection_ordered_and_empty() {
        let sel = Selection {
            start: (2, 3),
            end: (1, 1),
        };
        let ordered = sel.ordered();
        assert_eq!(ordered.0, (1, 1));
        assert!(!sel.is_empty());
        let empty = Selection {
            start: (0, 0),
            end: (0, 0),
        };
        assert!(empty.is_empty());
    }

    #[test]
    fn line_width_counts_spans() {
        let line = Line::from(vec![Span::raw("abc"), Span::raw("中")]);
        assert!(line_width(&line) >= 4);
    }

    #[test]
    fn slice_line_by_cols_extracts() {
        assert_eq!(slice_line_by_cols("hello", 1, 4), "ell");
        assert_eq!(slice_line_by_cols("hello", 3, 3), "");
    }

    #[test]
    fn extract_selection_spans_lines() {
        let lines = vec!["hello".to_string(), "world".to_string()];
        let sel = Selection {
            start: (0, 2),
            end: (1, 3),
        };
        let out = extract_selection(&lines, sel);
        assert!(out.contains("llo"));
        assert!(out.contains("wor"));
    }

    #[test]
    fn apply_selection_marks_text() {
        let text = Text::from(vec![Line::from("hello"), Line::from("world")]);
        let sel = Selection {
            start: (0, 1),
            end: (0, 3),
        };
        let styled = apply_selection_to_text(&text, 0, sel, Style::default());
        assert_eq!(styled.lines.len(), 2);
    }

    #[test]
    fn chat_position_from_mouse_bounds() {
        let text = Text::from(vec![Line::from("hello")]);
        let inner = Rect::new(0, 0, 10, 2);
        let pos = chat_position_from_mouse(&text, 0, inner, 2, 0);
        assert_eq!(pos.0, 0);
    }
}
