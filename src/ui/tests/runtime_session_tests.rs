#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::conversation::ConversationData;
    use crate::llm::prompts::{PromptRegistry, SystemPrompt};
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::session::SessionData;
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::events::RuntimeEvent;
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_session::{
        fork_last_tab_for_retry, restore_tabs_from_session, spawn_preheat_workers,
    };
    use std::fs;
    use std::sync::mpsc;

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
                content: "sys".to_string(),
            }],
        }
    }

    fn args() -> Args {
        Args {
            model: "m".to_string(),
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

    #[test]
    fn restore_tabs_from_session_loads_conversations() {
        let _guard = env_lock().lock().unwrap();
        let temp = std::env::temp_dir().join("deepchat-restore-session");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let prev = set_env("HOME", &temp.to_string_lossy());
        let conv = ConversationData {
            id: "c1".to_string(),
            category: "默认".to_string(),
            messages: Vec::new(),
            model_key: Some("m1".to_string()),
            prompt_key: Some("p1".to_string()),
            code_exec_container_id: None,
        };
        crate::conversation::save_conversation(&conv).unwrap();
        let session = SessionData {
            id: "s1".to_string(),
            categories: vec!["默认".to_string()],
            active_category: "默认".to_string(),
            open_conversations: vec!["c1".to_string()],
            active_conversation: Some("c1".to_string()),
        };
        let (tabs, active_tab, categories, active_category) =
            restore_tabs_from_session(&session, &registry(), &prompt_registry(), &args()).unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(active_tab, 0);
        assert_eq!(categories[active_category], "默认");
        restore_env("HOME", prev);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn fork_last_tab_for_retry_creates_new_tab() {
        let mut tabs = vec![TabState::new(
            "c1".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        tabs[0].app.messages.push(crate::types::Message {
            role: crate::types::ROLE_USER.to_string(),
            content: "hi".to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        let mut active_tab = 0usize;
        let res = fork_last_tab_for_retry(
            &mut tabs,
            &mut active_tab,
            &registry(),
            &prompt_registry(),
            &args(),
        );
        assert!(res.is_some());
        assert!(tabs.len() > 1);
    }

    #[test]
    fn spawn_preheat_workers_processes_tasks() {
        let (task_tx, task_rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();
        spawn_preheat_workers(task_rx, res_tx);
        task_tx.send(sample_preheat_task()).unwrap();
        let result = res_rx.recv().unwrap();
        let RuntimeEvent::Preheat(result) = result else {
            panic!("unexpected runtime event");
        };
        assert_eq!(result.idx, 0);
    }

    fn sample_preheat_task() -> crate::ui::runtime_helpers::PreheatTask {
        crate::ui::runtime_helpers::PreheatTask {
            tab: 0,
            idx: 0,
            msg: crate::types::Message {
                role: crate::types::ROLE_ASSISTANT.to_string(),
                content: "hi".to_string(),
                tool_call_id: None,
                tool_calls: None,
            },
            width: 40,
            theme: crate::render::RenderTheme {
                bg: ratatui::style::Color::Black,
                fg: Some(ratatui::style::Color::White),
                code_bg: ratatui::style::Color::Black,
                code_theme: "base16-ocean.dark",
                heading_fg: Some(ratatui::style::Color::Cyan),
            },
            streaming: false,
        }
    }
}
