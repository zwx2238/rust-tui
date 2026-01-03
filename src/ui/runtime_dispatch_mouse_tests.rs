#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext, handle_mouse_event_loop};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_view::ViewState;
    use crossterm::event::{MouseEvent, MouseEventKind};
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
            wait_gdb: false,
        }
    }

    fn layout() -> LayoutContext {
        LayoutContext {
            size: Rect::new(0, 0, 80, 24),
            tabs_area: Rect::new(0, 1, 80, 1),
            msg_area: Rect::new(0, 2, 80, 18),
            input_area: Rect::new(0, 20, 80, 3),
            category_area: Rect::new(0, 1, 10, 5),
            view_height: 10,
            total_lines: 0,
        }
    }

    struct CtxParams<'a> {
        tabs: &'a mut Vec<TabState>,
        active_tab: &'a mut usize,
        categories: &'a mut Vec<String>,
        active_category: &'a mut usize,
        theme: &'a RenderTheme,
        registry: &'a ModelRegistry,
        prompt_registry: &'a PromptRegistry,
        args: &'a Args,
    }

    fn ctx<'a>(params: CtxParams<'a>) -> DispatchContext<'a> {
        DispatchContext {
            tabs: params.tabs,
            active_tab: params.active_tab,
            categories: params.categories,
            active_category: params.active_category,
            msg_width: 40,
            theme: params.theme,
            registry: params.registry,
            prompt_registry: params.prompt_registry,
            args: params.args,
        }
    }

    #[test]
    fn mouse_scroll_in_overlay_does_not_panic() {
        let mut tabs = vec![TabState::new(
            "id".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::Summary);
        let mut ctx = ctx(CtxParams {
            tabs: &mut tabs,
            active_tab: &mut active_tab,
            categories: &mut categories,
            active_category: &mut active_category,
            theme: &theme,
            registry: &registry,
            prompt_registry: &prompt_registry,
            args: &args,
        });
        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 5,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        handle_mouse_event_loop(mouse, &mut ctx, layout(), &mut view, &[]);
    }
}
