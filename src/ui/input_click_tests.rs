#[cfg(test)]
mod tests {
    use crate::ui::input_click::{click_to_cursor, update_input_view_top};
    use crate::ui::state::App;
    use ratatui::layout::Rect;

    fn app() -> App {
        App::new("", "m1", "p1")
    }

    #[test]
    fn update_input_view_top_scrolls_to_cursor() {
        let mut app = app();
        app.input.insert_str("a\nb\nc");
        let area = Rect::new(0, 0, 10, 5);
        let (row, _) = app.input.cursor();
        let inner = crate::ui::draw::input_inner_area(area);
        let height = inner.height.max(1) as u16;
        let prev_top = app.input_view_top_row;
        update_input_view_top(&mut app, area);
        let expected = if (row as u16) < prev_top {
            row as u16
        } else if prev_top + height <= row as u16 {
            row as u16 + 1 - height
        } else {
            prev_top
        };
        assert_eq!(app.input_view_top_row, expected);
    }

    #[test]
    fn click_to_cursor_maps_position() {
        let mut app = app();
        app.input.insert_str("ab\ncd");
        let area = Rect::new(0, 0, 10, 5);
        let inner = crate::ui::draw::input_inner_area(area);
        let (row, col) = click_to_cursor(&app, area, inner.x + 1, inner.y);
        assert_eq!((row, col), (0, 1));
    }

    #[test]
    fn click_to_cursor_outside_returns_end() {
        let mut app = app();
        app.input.insert_str("ab\ncd");
        let area = Rect::new(0, 0, 10, 5);
        let (row, col) = click_to_cursor(&app, area, 0, 0);
        assert_eq!((row, col), (1, 2));
    }
}
