#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::ui::runtime_dispatch::{DispatchContext, LayoutContext, handle_key_event_loop};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_view::ViewState;
    use crate::ui::state::RequestHandle;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use std::sync::{Arc, atomic::AtomicBool};

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

    fn ctx<'a>(
        tabs: &'a mut Vec<TabState>,
        active_tab: &'a mut usize,
        categories: &'a mut Vec<String>,
        active_category: &'a mut usize,
        theme: &'a RenderTheme,
        registry: &'a ModelRegistry,
        prompt_registry: &'a PromptRegistry,
        args: &'a Args,
    ) -> DispatchContext<'a> {
        DispatchContext {
            tabs,
            active_tab,
            categories,
            active_category,
            msg_width: 40,
            theme,
            registry,
            prompt_registry,
            args,
        }
    }

    #[test]
    fn ctrl_q_exits() {
        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::Summary);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let should_exit = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert!(should_exit);
    }

    #[test]
    fn global_shortcuts_switch_category_and_tabs() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::Summary);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert_eq!(*ctx.active_category, 1);
        let key = KeyEvent::new(KeyCode::F(8), KeyModifiers::NONE);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert_eq!(*ctx.active_tab, 1);
    }

    #[test]
    fn ctrl_t_creates_new_tab() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::Summary);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert_eq!(ctx.tabs.len(), 2);
    }

    #[test]
    fn f6_stops_active_request() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        let cancel = Arc::new(AtomicBool::new(false));
        tabs[0].app.active_request = Some(RequestHandle { id: 1, cancel: cancel.clone() });
        tabs[0].app.busy = true;
        tabs[0].app.pending_assistant = Some(0);
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let key = KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert!(cancel.load(std::sync::atomic::Ordering::Relaxed));
        assert!(!ctx.tabs[0].app.busy);
    }

    #[test]
    fn f5_pushes_prompt_locked_notice() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let key = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert!(ctx.tabs[0].app.notice.is_some());
    }

    #[test]
    fn code_exec_reason_escape_clears_target() {
        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        tabs[0].app.code_exec_reason_target =
            Some(crate::ui::state::CodeExecReasonTarget::Deny);
        tabs[0].app.code_exec_reason_input = tui_textarea::TextArea::default();
        tabs[0].app.code_exec_reason_input.insert_str("why");
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let mut view = ViewState::new();
        view.overlay.open(crate::ui::overlay::OverlayKind::CodeExec);
        let mut ctx = ctx(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            &theme,
            &registry,
            &prompt_registry,
            &args,
        );
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let _ = handle_key_event_loop(key, &mut ctx, layout(), &mut view, &[]).unwrap();
        assert!(ctx.tabs[0].app.code_exec_reason_target.is_none());
        assert!(ctx.tabs[0].app.code_exec_reason_input.lines().join("").is_empty());
    }
}
