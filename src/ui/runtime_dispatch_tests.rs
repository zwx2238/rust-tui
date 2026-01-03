#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::ui::runtime_dispatch::{
        DispatchContext, apply_model_selection, apply_prompt_selection, can_change_prompt,
        cycle_model, fork_message_by_index, fork_message_into_new_tab, resolve_model,
        start_pending_request, sync_model_selection, sync_prompt_selection,
    };
    use crate::ui::runtime_helpers::TabState;
    use ratatui::style::Color;

    struct DispatchTestState {
        tabs: Vec<TabState>,
        active_tab: usize,
        categories: Vec<String>,
        active_category: usize,
        theme: RenderTheme,
        registry: ModelRegistry,
        prompt_registry: PromptRegistry,
        args: Args,
    }

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
            prompts: vec![
                SystemPrompt {
                    key: "p1".to_string(),
                    content: "sys1".to_string(),
                },
                SystemPrompt {
                    key: "p2".to_string(),
                    content: "sys2".to_string(),
                },
            ],
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

    fn base_state() -> DispatchTestState {
        DispatchTestState {
            tabs: vec![TabState::new(
                "id".into(),
                "cat".into(),
                "",
                false,
                "m1",
                "p1",
            )],
            active_tab: 0,
            categories: vec!["cat".to_string()],
            active_category: 0,
            theme: theme(),
            registry: registry(),
            prompt_registry: prompt_registry(),
            args: args(),
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

    fn ctx_from_state<'a>(state: &'a mut DispatchTestState) -> DispatchContext<'a> {
        ctx(CtxParams {
            tabs: &mut state.tabs,
            active_tab: &mut state.active_tab,
            categories: &mut state.categories,
            active_category: &mut state.active_category,
            theme: &state.theme,
            registry: &state.registry,
            prompt_registry: &state.prompt_registry,
            args: &state.args,
        })
    }

    fn layout() -> crate::ui::runtime_dispatch::LayoutContext {
        crate::ui::runtime_dispatch::LayoutContext {
            size: ratatui::layout::Rect::new(0, 0, 80, 24),
            tabs_area: ratatui::layout::Rect::new(0, 1, 80, 1),
            msg_area: ratatui::layout::Rect::new(0, 2, 80, 18),
            input_area: ratatui::layout::Rect::new(0, 20, 80, 3),
            category_area: ratatui::layout::Rect::new(0, 1, 10, 5),
            view_height: 10,
            total_lines: 0,
        }
    }

    #[test]
    fn can_change_prompt_checks_user_messages() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m1", "p1");
        assert!(can_change_prompt(&tab.app));
        tab.app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        assert!(!can_change_prompt(&tab.app));
    }

    #[test]
    fn apply_model_selection_updates_key() {
        let mut state = base_state();
        let mut ctx = ctx_from_state(&mut state);
        apply_model_selection(&mut ctx, 0);
        assert_eq!(ctx.tabs[0].app.model_key, "m1");
    }

    #[test]
    fn apply_prompt_selection_updates_system_prompt() {
        let mut state = base_state();
        let mut ctx = ctx_from_state(&mut state);
        apply_prompt_selection(&mut ctx, 1);
        assert_eq!(ctx.tabs[0].app.prompt_key, "p2");
    }

    #[test]
    fn fork_message_requires_user_message() {
        let mut state = base_state();
        state.tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let mut ctx = ctx_from_state(&mut state);
        assert!(!fork_message_by_index(&mut ctx, 0));
    }

    #[test]
    fn fork_message_creates_new_tab() {
        let mut state = base_state();
        state.tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let jump_rows = vec![crate::ui::jump::JumpRow {
            index: 1,
            role: crate::types::ROLE_USER.to_string(),
            preview: "hi".to_string(),
            scroll: 0,
        }];
        let mut ctx = ctx_from_state(&mut state);
        assert!(fork_message_into_new_tab(&mut ctx, &jump_rows, 0));
        assert!(ctx.tabs.len() > 1);
    }

    #[test]
    fn cycle_model_wraps() {
        let mut registry = registry();
        registry.models.push(ModelProfile {
            key: "m2".to_string(),
            base_url: "http://example.com".to_string(),
            api_key: "k".to_string(),
            model: "model2".to_string(),
        });
        let mut key = "m1".to_string();
        cycle_model(&registry, &mut key);
        assert_eq!(key, "m2");
        cycle_model(&registry, &mut key);
        assert_eq!(key, "m1");
    }

    #[test]
    fn resolve_model_falls_back_to_default() {
        let registry = registry();
        let model = resolve_model(&registry, "missing");
        assert_eq!(model.key, "m1");
    }

    #[test]
    fn fork_message_by_index_non_user_pushes_notice() {
        let mut state = base_state();
        state.tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_ASSISTANT.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let mut ctx = ctx_from_state(&mut state);
        assert!(!fork_message_by_index(&mut ctx, 0));
        assert!(ctx.tabs[0].app.notice.is_some());
    }

    #[test]
    fn start_pending_request_uses_pending_send() {
        let registry = registry();
        let args = args();
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m1", "p1");
        tab.app.pending_send = Some("hello".to_string());
        let mut tabs = [tab];
        start_pending_request(&registry, &args, &tx, 0, &mut tabs[0]);
        assert!(
            tabs[0]
                .app
                .messages
                .iter()
                .any(|m| m.role == crate::types::ROLE_USER)
        );
    }

    #[test]
    fn sync_selections_clamp_view() {
        let mut state = base_state();
        let ctx = ctx_from_state(&mut state);
        let layout = layout();
        let mut view = crate::ui::runtime_view::ViewState::new();
        sync_model_selection(&mut view, &ctx, layout);
        sync_prompt_selection(&mut view, &ctx, layout);
        assert_eq!(view.model.selected, 0);
        assert_eq!(view.prompt.selected, 0);
    }
}
