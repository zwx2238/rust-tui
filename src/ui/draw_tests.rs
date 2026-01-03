#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::draw::{draw_categories, draw_footer, draw_header, draw_tabs};
    use crate::ui::draw::draw_messages;
    use crate::ui::draw_input::draw_input;
    use crate::ui::selection::Selection;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui::text::{Line, Text};
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
    fn draw_header_footer_tabs_categories() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let labels = vec![" 对话 1 ".to_string(), " 对话 2 ".to_string()];
        let categories = vec!["默认".to_string(), "分类 2".to_string()];
        terminal
            .draw(|f| {
                draw_header(f, Rect::new(0, 0, 80, 1), &theme(), Some("note"));
                draw_footer(f, Rect::new(0, 23, 80, 1), &theme(), false);
                draw_tabs(
                    f,
                    Rect::new(0, 2, 80, 1),
                    &labels,
                    0,
                    &theme(),
                    None,
                );
                draw_categories(
                    f,
                    Rect::new(0, 3, 12, 10),
                    &categories,
                    1,
                    &theme(),
                );
            })
            .unwrap();
    }

    #[test]
    fn draw_messages_and_input() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let text = Text::from(vec![Line::from("line1"), Line::from("line2")]);
        let selection = Selection {
            start: (0, 1),
            end: (0, 3),
        };
        let mut input = tui_textarea::TextArea::default();
        input.insert_str("hello");
        terminal
            .draw(|f| {
                draw_messages(
                    f,
                    Rect::new(0, 4, 80, 15),
                    &text,
                    0,
                    &theme(),
                    true,
                    2,
                    Some(selection),
                );
                draw_input(
                    f,
                    Rect::new(0, 20, 80, 3),
                    &mut input,
                    &theme(),
                    true,
                    false,
                    "model",
                    "prompt",
                );
            })
            .unwrap();
    }
}
