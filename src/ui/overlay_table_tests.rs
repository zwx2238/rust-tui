#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::overlay_table::{
        OverlayTable, centered_area, draw_overlay_table, header_style, row_at, visible_rows,
    };
    use ratatui::backend::TestBackend;
    use ratatui::layout::{Constraint, Rect};
    use ratatui::style::Color;
    use ratatui::text::Line;
    use ratatui::widgets::{Cell, Row};
    use ratatui::Terminal;

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    #[test]
    fn computes_centered_area() {
        let area = Rect::new(0, 0, 100, 40);
        let centered = centered_area(area, 80, 5, 20);
        assert!(centered.width <= area.width);
        assert!(centered.height <= area.height);
    }

    #[test]
    fn row_at_bounds() {
        let area = Rect::new(0, 0, 10, 6);
        assert_eq!(row_at(area, 5, 0, 1, 2), Some(0));
        assert_eq!(row_at(area, 5, 0, 1, 1), None);
        assert_eq!(row_at(area, 2, 0, 1, 10), None);
    }

    #[test]
    fn visible_rows_minimum() {
        let area = Rect::new(0, 0, 10, 3);
        assert_eq!(visible_rows(area), 1);
    }

    #[test]
    fn draws_overlay_table() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let header = Row::new(vec![Cell::from("A"), Cell::from("B")]).style(header_style(&theme()));
        let rows = vec![Row::new(vec![Cell::from("1"), Cell::from("2")])];
        let table = OverlayTable {
            title: Line::from("Title"),
            header,
            rows,
            widths: vec![Constraint::Length(5), Constraint::Length(5)],
            selected: 0,
            scroll: 0,
            theme: &theme(),
        };
        terminal
            .draw(|f| {
                draw_overlay_table(f, Rect::new(0, 0, 40, 10), table);
            })
            .unwrap();
    }
}
