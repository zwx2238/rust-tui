#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::session::SessionLocation;
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_loop_helpers::{HandlePendingCommandParams, handle_pending_command};
    use crate::ui::state::PendingCommand;
    use std::fs;
    use std::sync::mpsc;

    struct PendingCtx {
        tabs: Vec<TabState>,
        active_tab: usize,
        categories: Vec<String>,
        active_category: usize,
        session_location: Option<SessionLocation>,
    }

    fn registry() -> ModelRegistry {
        ModelRegistry {
            default_key: "m1".to_string(),
            models: vec![ModelProfile {
                key: "m1".to_string(),
                base_url: "http://example.com".to_string(),
                api_key: "".to_string(),
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
            workspace: "/tmp/deepchat-workspace".to_string(),
            yolo: false,
            read_only: false,
            wait_gdb: false,
        }
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

    fn base_pending_ctx() -> PendingCtx {
        PendingCtx {
            tabs: vec![TabState::new(
                "id".into(),
                "默认".into(),
                "",
                false,
                "m1",
                "p1",
            )],
            active_tab: 0,
            categories: vec!["默认".to_string()],
            active_category: 0,
            session_location: None,
        }
    }

    fn run_pending(
        ctx: &mut PendingCtx,
        pending: PendingCommand,
        registry: &ModelRegistry,
        prompt_registry: &PromptRegistry,
        args: &Args,
        tx: &mpsc::Sender<crate::ui::net::UiEvent>,
    ) {
        handle_pending_command(HandlePendingCommandParams {
            tabs: &mut ctx.tabs,
            active_tab: &mut ctx.active_tab,
            categories: &mut ctx.categories,
            active_category: &mut ctx.active_category,
            pending,
            session_location: &mut ctx.session_location,
            registry,
            prompt_registry,
            args,
            tx,
        });
    }

    fn ctx_with_existing_conversation() -> PendingCtx {
        PendingCtx {
            tabs: vec![
                TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
                TabState::new("conv1".into(), "分类 2".into(), "", false, "m1", "p1"),
            ],
            active_tab: 0,
            categories: vec!["默认".to_string(), "分类 2".to_string()],
            active_category: 0,
            session_location: None,
        }
    }

    #[test]
    fn handle_pending_command_save_session_reports_success() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-save-session");
        let mut ctx = base_pending_ctx();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        run_pending(
            &mut ctx,
            PendingCommand::SaveSession,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(
            ctx.tabs[0]
                .app
                .messages
                .iter()
                .any(|m| m.content.contains("已保存会话"))
        );

        cleanup_temp_home(temp, prev);
    }

    #[test]
    fn handle_pending_command_save_session_reports_error() {
        let _guard = env_lock().lock().unwrap();
        let prev = std::env::var("HOME").ok();
        restore_env("HOME", None);

        let mut ctx = base_pending_ctx();
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        run_pending(
            &mut ctx,
            PendingCommand::SaveSession,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(
            ctx.tabs[0]
                .app
                .messages
                .iter()
                .any(|m| m.content.contains("保存失败"))
        );

        restore_env("HOME", prev);
    }

    #[test]
    fn open_conversation_switches_to_existing_tab() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-open-existing");

        let mut ctx = ctx_with_existing_conversation();
        ctx.tabs[0].app.pending_open_conversation = Some("conv1".to_string());
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        run_pending(
            &mut ctx,
            PendingCommand::OpenConversation,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert_eq!(ctx.active_tab, 1);
        assert_eq!(ctx.active_category, 1);

        cleanup_temp_home(temp, prev);
    }

    #[test]
    fn open_conversation_reports_error_on_missing_file() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-open-missing");
        let mut ctx = base_pending_ctx();
        ctx.tabs[0].app.pending_open_conversation = Some("missing".to_string());
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        run_pending(
            &mut ctx,
            PendingCommand::OpenConversation,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(
            ctx.tabs[0]
                .app
                .messages
                .iter()
                .any(|m| m.content.contains("打开对话失败"))
        );

        cleanup_temp_home(temp, prev);
    }

    #[test]
    fn new_category_uses_default_name_when_empty() {
        let _guard = env_lock().lock().unwrap();
        let (temp, prev) = setup_temp_home("deepchat-new-category");
        let mut ctx = base_pending_ctx();
        ctx.tabs = vec![TabState::new(
            "id".into(),
            "默认".into(),
            "",
            false,
            "missing",
            "missing",
        )];
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        run_pending(
            &mut ctx,
            PendingCommand::NewCategory,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(ctx.categories.iter().any(|c| c == "分类 1"));
        assert_eq!(ctx.active_tab, 1);

        cleanup_temp_home(temp, prev);
    }
}
