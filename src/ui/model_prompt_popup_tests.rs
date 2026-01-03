#[cfg(test)]
mod tests {
    use crate::llm::prompts::SystemPrompt;
    use crate::model_registry::ModelProfile;
    use crate::render::RenderTheme;
    use crate::ui::model_popup::draw_model_popup;
    use crate::ui::prompt_popup::draw_prompt_popup;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
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
    fn draw_model_popup_smoke() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let models = vec![ModelProfile {
            key: "m1".to_string(),
            base_url: "http://example.com".to_string(),
            api_key: "k".to_string(),
            model: "model".to_string(),
        }];
        terminal
            .draw(|f| {
                draw_model_popup(f, Rect::new(0, 0, 60, 20), &models, 0, 0, &theme());
            })
            .unwrap();
    }

    #[test]
    fn draw_prompt_popup_smoke() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let prompts = vec![SystemPrompt {
            key: "p1".to_string(),
            content: "sys".to_string(),
        }];
        terminal
            .draw(|f| {
                draw_prompt_popup(f, Rect::new(0, 0, 60, 20), &prompts, 0, 0, &theme());
            })
            .unwrap();
    }
}
