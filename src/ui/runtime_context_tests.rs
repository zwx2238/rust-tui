#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::ui::runtime_context::{make_dispatch_context, make_layout_context};
    use crate::ui::runtime_helpers::TabState;
    use ratatui::layout::Rect;
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

    fn registry() -> ModelRegistry {
        ModelRegistry {
            default_key: "m1".to_string(),
            models: vec![ModelProfile {
                key: "m1".to_string(),
                base_url: "http://example.com".to_string(),
                api_key: "k".to_string(),
                model: "model".to_string(),
            }],
        }
    }

    fn prompt_registry() -> PromptRegistry {
        PromptRegistry {
            default_key: "p1".to_string(),
            prompts: vec![SystemPrompt {
                key: "p1".to_string(),
                content: "sys1".to_string(),
            }],
        }
    }

    fn args() -> Args {
        Args {
            model: "m".to_string(),
            system: "sys".to_string(),
            base_url: "http://example.com".to_string(),
            show_reasoning: false,
            config: None,
            resume: None,
            replay_fork_last: false,
            enable: None,
            log_requests: None,
            perf: false,
            question_set: None,
            yolo: false,
            read_only: false,
        }
    }

    #[test]
    fn make_dispatch_context_wires_fields() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let ctx = make_dispatch_context(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            40,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        assert_eq!(ctx.msg_width, 40);
        assert_eq!(ctx.tabs.len(), 1);
    }

    #[test]
    fn make_layout_context_maps_fields() {
        let layout = make_layout_context(
            Rect::new(0, 0, 80, 24),
            Rect::new(0, 1, 80, 1),
            Rect::new(0, 2, 80, 10),
            Rect::new(0, 12, 80, 3),
            Rect::new(0, 1, 10, 5),
            10,
            20,
        );
        assert_eq!(layout.view_height, 10);
        assert_eq!(layout.total_lines, 20);
    }
}
