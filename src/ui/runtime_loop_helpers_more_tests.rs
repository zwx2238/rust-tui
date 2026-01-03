#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_loop_helpers::handle_pending_command;
    use crate::ui::state::PendingCommand;
    use std::fs;
    use std::sync::mpsc;

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
            yolo: false,
            read_only: false,
        }
    }

    fn set_home(temp: &std::path::Path) -> Option<String> {
        set_env("HOME", &temp.to_string_lossy())
    }

    fn restore_home(prev: Option<String>) {
        restore_env("HOME", prev);
    }

    #[test]
    fn handle_pending_command_save_session_reports_success() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-save-session");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let mut session_location = None;
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        handle_pending_command(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            PendingCommand::SaveSession,
            &mut session_location,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(tabs[0]
            .app
            .messages
            .iter()
            .any(|m| m.content.contains("已保存会话")));

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn handle_pending_command_save_session_reports_error() {
        let _guard = env_lock().lock().unwrap();
        let prev = std::env::var("HOME").ok();
        restore_env("HOME", None);

        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let mut session_location = None;
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        handle_pending_command(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            PendingCommand::SaveSession,
            &mut session_location,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(tabs[0]
            .app
            .messages
            .iter()
            .any(|m| m.content.contains("保存失败")));

        restore_env("HOME", prev);
    }

    #[test]
    fn open_conversation_switches_to_existing_tab() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-open-existing");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

        let mut tabs = vec![
            TabState::new("id1".into(), "默认".into(), "", false, "m1", "p1"),
            TabState::new("conv1".into(), "分类 2".into(), "", false, "m1", "p1"),
        ];
        tabs[0].app.pending_open_conversation = Some("conv1".to_string());
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string(), "分类 2".to_string()];
        let mut active_category = 0usize;
        let mut session_location = None;
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        handle_pending_command(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            PendingCommand::OpenConversation,
            &mut session_location,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert_eq!(active_tab, 1);
        assert_eq!(active_category, 1);

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn open_conversation_reports_error_on_missing_file() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-open-missing");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

        let mut tabs = vec![TabState::new("id".into(), "默认".into(), "", false, "m1", "p1")];
        tabs[0].app.pending_open_conversation = Some("missing".to_string());
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let mut session_location = None;
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        handle_pending_command(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            PendingCommand::OpenConversation,
            &mut session_location,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(tabs[0]
            .app
            .messages
            .iter()
            .any(|m| m.content.contains("打开对话失败")));

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn new_category_uses_default_name_when_empty() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-new-category");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_home(&temp);

        let mut tabs = vec![TabState::new(
            "id".into(),
            "默认".into(),
            "",
            false,
            "missing",
            "missing",
        )];
        let mut active_tab = 0usize;
        let mut categories = vec!["默认".to_string()];
        let mut active_category = 0usize;
        let mut session_location = None;
        let registry = registry();
        let prompt_registry = prompt_registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        handle_pending_command(
            &mut tabs,
            &mut active_tab,
            &mut categories,
            &mut active_category,
            PendingCommand::NewCategory,
            &mut session_location,
            &registry,
            &prompt_registry,
            &args,
            &tx,
        );
        assert!(categories.iter().any(|c| c == "分类 1"));
        assert_eq!(active_tab, 1);

        restore_home(prev);
        let _ = fs::remove_dir_all(&temp);
    }
}
