#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext, handle_key_event_loop};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_view::ViewState;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

    struct KeyTestState {
        tabs: Vec<TabState>,
        active_tab: usize,
        categories: Vec<String>,
        active_category: usize,
        theme: RenderTheme,
        registry: ModelRegistry,
        prompt_registry: PromptRegistry,
        args: Args,
        view: ViewState,
    }

    fn key_test_state(tabs: Vec<TabState>, active_tab: usize) -> KeyTestState {
        KeyTestState {
            tabs,
            active_tab,
            categories: vec!["默认".to_string()],
            active_category: 0,
            theme: theme(),
            registry: registry(),
            prompt_registry: prompt_registry(),
            args: args(),
            view: ViewState::new(),
        }
    }

    fn ctx_and_view<'a>(state: &'a mut KeyTestState) -> (DispatchContext<'a>, &'a mut ViewState) {
        let ctx = ctx(CtxParams {
            tabs: &mut state.tabs,
            active_tab: &mut state.active_tab,
            categories: &mut state.categories,
            active_category: &mut state.active_category,
            theme: &state.theme,
            registry: &state.registry,
            prompt_registry: &state.prompt_registry,
            args: &state.args,
        });
        (ctx, &mut state.view)
    }

    #[test]
    fn ctrl_shift_w_closes_all_tabs() {
        let tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut state = key_test_state(tabs, 1);
        let (mut ctx, view) = ctx_and_view(&mut state);
        let key = KeyEvent::new(
            KeyCode::Char('w'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        let _ = handle_key_event_loop(key, &mut ctx, layout(), view, &[]).unwrap();
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(*ctx.active_tab, 0);
    }

    #[test]
    fn ctrl_o_closes_other_tabs() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id3".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 1usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
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
        let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(*ctx.active_tab, 0);
    }

    #[test]
    fn ctrl_w_closes_tab() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
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
        let key = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert_eq!(ctx.tabs.len(), 1);
    }

    #[test]
    fn f9_moves_to_next_tab() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
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
        let key = KeyEvent::new(KeyCode::F(9), KeyModifiers::NONE);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert_eq!(*ctx.active_tab, 1);
    }
}
