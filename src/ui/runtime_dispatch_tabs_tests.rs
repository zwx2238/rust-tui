#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::render::RenderTheme;
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::runtime_dispatch::{
        DispatchContext, close_all_tabs, close_other_tabs, close_tab, new_tab, next_category,
        next_tab, prev_category, prev_tab,
    };
    use crate::ui::runtime_helpers::TabState;
    use ratatui::style::Color;
    use std::fs;

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

    fn set_home(temp: &std::path::Path) -> Option<String> {
        set_env("HOME", &temp.to_string_lossy())
    }

    fn restore_home(prev: Option<String>) {
        restore_env("HOME", prev);
    }

    #[test]
    fn new_tab_adds_category_and_inherits_app_fields() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-tabs-new");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

        let mut tabs = vec![TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.prompts_dir = "/tmp/prompts".to_string();
        tabs[0].app.tavily_api_key = "key".to_string();
        tabs[0].app.set_log_session_id("session");
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
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
        new_tab(&mut ctx);
        assert_eq!(ctx.tabs.len(), 2);
        assert_eq!(ctx.tabs[1].app.prompts_dir, "/tmp/prompts");
        assert_eq!(ctx.tabs[1].app.tavily_api_key, "key");
        assert_eq!(ctx.tabs[1].app.log_session_id, "session");

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn close_tab_updates_active_tab() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-tabs-close");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 1usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
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
        close_tab(&mut ctx);
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(*ctx.active_tab, 0);

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn close_other_tabs_keeps_active_only() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-tabs-close-other");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 1usize;
        let mut categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut active_category = 1usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
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
        close_other_tabs(&mut ctx);
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(ctx.categories.len(), 1);
        assert_eq!(ctx.categories[0], "分类 2");
        assert_eq!(*ctx.active_tab, 0);
        assert_eq!(*ctx.active_category, 0);

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn close_all_tabs_resets_to_single_tab() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-tabs-close-all");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

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
        close_all_tabs(&mut ctx);
        assert_eq!(ctx.tabs.len(), 1);
        assert_eq!(ctx.categories.len(), 1);
        assert_eq!(*ctx.active_tab, 0);

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn tab_navigation_respects_category_visibility() {
        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("id2".into(), "分类 2".into(), "", false, "m1", "p1"),
            TabState::new("id3".into(), "默认".into(), "", false, "m1", "p1"),
        ];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut active_category = 0usize;
        let theme = theme();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
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
