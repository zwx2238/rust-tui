#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::runtime_dispatch::DispatchContext;
    use crate::ui::runtime_dispatch::tabs::{
        close_all_tabs, close_other_tabs, close_tab, new_tab, next_category, next_tab,
        prev_category, prev_tab,
    };
    use crate::ui::runtime_helpers::TabState;
    use ratatui::style::Color;
    use std::fs;

    struct TabsState {
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
                max_tokens: None,
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
            model: None,
            system: "sys".to_string(),
            base_url: "http://example.com".to_string(),
            show_reasoning: false,
            resume: None,
            replay_fork_last: false,
            enable: None,
            log_requests: None,
            perf: false,
            question_set: None,
            workspace: "/tmp/deepchat-workspace".to_string(),
            yolo: false,
            read_only: false,
            wait_gdb: false,
        }
    }

    fn base_state() -> TabsState {
        TabsState {
            tabs: vec![TabState::new(
                "id1".into(),
                "默认".into(),
                "",
                false,
                "m1",
                "p1",
            )],
            active_tab: 0,
            categories: vec!["默认".to_string()],
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

    fn ctx_from_state<'a>(state: &'a mut TabsState) -> DispatchContext<'a> {
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

    fn set_home(temp: &std::path::Path) -> Option<String> {
        set_env("HOME", &temp.to_string_lossy())
    }

    fn restore_home(prev: Option<String>) {
        restore_env("HOME", prev);
    }

    fn setup_temp_home(name: &str) -> (std::path::PathBuf, Option<String>) {
        let temp = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);
        (temp, prev)
    }

    fn cleanup_temp_home(path: std::path::PathBuf, prev: Option<String>) {
        restore_home(prev);
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn new_tab_adds_category_and_inherits_app_fields() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-tabs-new");
        let mut state = base_state();
        state.tabs[0].app.prompts_dir = "/tmp/prompts".to_string();
        state.tabs[0].app.tavily_api_key = "key".to_string();
        state.tabs[0].app.set_log_session_id("session");
        let mut ctx = ctx_from_state(&mut state);
        new_tab(&mut ctx);
        assert_eq!(ctx.tabs.len(), 2);
        assert_eq!(ctx.tabs[1].app.prompts_dir, "/tmp/prompts");
        assert_eq!(ctx.tabs[1].app.tavily_api_key, "key");
        assert_eq!(ctx.tabs[1].app.log_session_id, "session");

        cleanup_temp_home(temp, prev);
    }

    #[test]
    fn close_tab_updates_active_tab() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-tabs-close");
        let mut state = base_state();
        state.tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        state.active_tab = 1;
        let mut ctx = ctx_from_state(&mut state);
        close_tab(&mut ctx);
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(*ctx.active_tab, 0);

        cleanup_temp_home(temp, prev);
    }

    #[test]
    fn close_other_tabs_keeps_active_only() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-tabs-close-other");
        let mut state = base_state();
        state.tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
        ];
        state.active_tab = 1;
        state.categories = vec!["默认".to_string(), "分类 2".to_string()];
        state.active_category = 1;
        let mut ctx = ctx_from_state(&mut state);
        close_other_tabs(&mut ctx);
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(ctx.categories.len(), 1);
        assert_eq!(ctx.categories[0], "分类 2");
        assert_eq!(*ctx.active_tab, 0);
        assert_eq!(*ctx.active_category, 0);

        cleanup_temp_home(temp, prev);
    }

    #[test]
    fn close_all_tabs_resets_to_single_tab() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-tabs-close-all");
        let mut state = base_state();
        state.tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
        ];
        state.categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut ctx = ctx_from_state(&mut state);
        close_all_tabs(&mut ctx);
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(ctx.categories.len(), 1);
        assert_eq!(*ctx.active_tab, 0);

        cleanup_temp_home(temp, prev);
    }

    #[test]
    fn tab_navigation_respects_category_visibility() {
        let mut state = base_state();
        state.tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
            TabState::new("id3".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        state.categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut ctx = ctx_from_state(&mut state);
        next_tab(&mut ctx);
        assert_eq!(*ctx.active_tab, 2);
        prev_tab(&mut ctx);
        assert_eq!(*ctx.active_tab, 0);
        next_category(&mut ctx);
        assert_eq!(*ctx.active_category, 1);
        assert_eq!(*ctx.active_tab, 1);
        prev_category(&mut ctx);
        assert_eq!(*ctx.active_category, 0);
        assert_eq!(*ctx.active_tab, 0);
    }
}
