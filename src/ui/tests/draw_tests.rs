#[cfg(test)]
mod tests {
    use crate::render::RenderTheme;
    use crate::ui::draw::draw_messages;
    use crate::ui::draw::{draw_categories, draw_footer, draw_header, draw_tabs};
    use crate::ui::draw_input::draw_input;
    use crate::ui::selection::Selection;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui::text::{Line, Text};

    fn theme() -> RenderTheme {
        RenderTheme {
            bg: Color::Black,
            fg: Some(Color::White),
            code_bg: Color::Black,
            code_theme: "base16-ocean.dark",
            heading_fg: Some(Color::Cyan),
        }
    }

    fn with_terminal<F>(f: F)
    where
        F: FnOnce(&mut ratatui::Frame<'_>),
    {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(f).unwrap();
    }

    #[test]
    fn draw_header_footer_tabs_categories() {
        let labels = vec![" 对话 1 ".to_string(), " 对话 2 ".to_string()];
        let categories = vec!["默认".to_string(), "分类 2".to_string()];
        with_terminal(|f| {
            draw_header(f, Rect::new(0, 0, 80, 1), &theme(), Some("note"));
            draw_footer(f, Rect::new(0, 23, 80, 1), &theme(), false, false);
            draw_tabs(f, Rect::new(0, 2, 80, 1), &labels, 0, &theme(), None);
            draw_categories(f, Rect::new(0, 3, 12, 10), &categories, 1, &theme());
        });
    }

    #[test]
    fn draw_messages_and_input() {
        let text = sample_text();
        let selection = sample_selection();
        let mut input = sample_input();
        with_terminal(|f| {
            draw_messages_and_input_frame(f, &text, selection, &mut input);
        });
    }

    fn sample_text() -> Text<'static> {
        Text::from(vec![Line::from("line1"), Line::from("line2")])
    }

    fn sample_selection() -> Selection {
        Selection {
            start: (0, 1),
            end: (0, 3),
        }
    }

    fn sample_input() -> tui_textarea::TextArea<'static> {
        let mut input = tui_textarea::TextArea::default();
        input.insert_str("hello");
        input
    }

    fn draw_messages_and_input_frame(
        f: &mut ratatui::Frame<'_>,
        text: &Text<'_>,
        selection: Selection,
        input: &mut tui_textarea::TextArea<'static>,
    ) {
        let theme = theme();
        draw_messages(f, messages_params(text, selection, &theme));
        draw_input(f, input_params(input, &theme));
    }

    fn messages_params<'a>(
        text: &'a Text<'a>,
        selection: Selection,
        theme: &'a RenderTheme,
    ) -> crate::ui::draw::MessagesDrawParams<'a> {
        crate::ui::draw::MessagesDrawParams {
            area: Rect::new(0, 4, 80, 15),
            text,
            scroll: 0,
            theme,
            focused: true,
            total_lines: 2,
            selection: Some(selection),
        }
    }

    fn input_params<'a>(
        input: &'a mut tui_textarea::TextArea<'static>,
        theme: &'a RenderTheme,
    ) -> crate::ui::draw_input::InputDrawParams<'a, 'static> {
        crate::ui::draw_input::InputDrawParams {
            area: Rect::new(0, 20, 80, 3),
            input,
            theme,
            focused: true,
            busy: false,
            model_key: "model",
            prompt_key: "prompt",
        }
    }
}
