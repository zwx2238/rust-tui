#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::shortcut_help::{draw_shortcut_help, help_popup_area, help_rows_len};
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use ratatui::style::Color;

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
    fn help_rows_non_empty() {
        assert!(help_rows_len() > 0);
    }

    #[test]
    fn help_popup_area_within_bounds() {
        let area = Rect::new(0, 0, 100, 40);
        let popup = help_popup_area(area, 10);
        assert!(popup.width <= area.width);
        assert!(popup.height <= area.height);
    }

    #[test]
    fn draws_help_popup() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let area = Rect::new(0, 0, 80, 24);
        terminal
            .draw(|f| {
                draw_shortcut_help(f, area, 0, 0, &theme());
            })
            .unwrap();
    }
}
