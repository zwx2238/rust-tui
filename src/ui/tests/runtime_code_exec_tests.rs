#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::types::{ToolCall, ToolFunctionCall};
    use crate::ui::runtime_code_exec::{
        handle_code_exec_approve, handle_code_exec_deny, handle_code_exec_exit,
        handle_code_exec_request, handle_code_exec_stop,
    };
    use crate::ui::runtime_helpers::TabState;
    use std::sync::mpsc;
    use std::sync::{Arc, atomic::AtomicBool};

    fn registry() -> ModelRegistry {
        ModelRegistry {
            default_key: "m1".to_string(),
            models: vec![ModelProfile {
                key: "m1".to_string(),
                base_url: "http://example.com".to_string(),
                api_key: "".to_string(),
                model: "model".to_string(),
                max_tokens: None,
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

    fn code_exec_call(args: &str) -> ToolCall {
        ToolCall {
            id: "call1".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "code_exec".to_string(),
                arguments: args.to_string(),
            },
        }
    }

    #[test]
    fn handle_code_exec_request_sets_pending() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let call = code_exec_call(r#"{"language":"python","code":"print(1)"}"#);
        handle_code_exec_request(&mut tab, &call).unwrap();
        assert!(tab.app.pending_code_exec.is_some());
        assert_eq!(tab.app.code_exec_scroll, 0);
    }

    #[test]
    fn handle_code_exec_request_rejects_invalid_json() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let call = code_exec_call("{");
        let err = handle_code_exec_request(&mut tab, &call).unwrap_err();
        assert!(err.contains("参数解析失败"));
    }

    #[test]
    fn handle_code_exec_stop_sets_reason() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        tab.app.code_exec_cancel = Some(Arc::new(AtomicBool::new(false)));
        handle_code_exec_stop(&mut tab);
        assert_eq!(
            tab.app
                .pending_code_exec
                .as_ref()
                .and_then(|p| p.stop_reason.clone())
                .as_deref(),
            Some("用户中止")
        );
    }

    #[test]
    fn handle_code_exec_exit_emits_tool_message() {
        let registry = registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        tab.app.code_exec_finished_output = Some("output".to_string());
        handle_code_exec_exit(&mut tab, 0, &registry, &args, &tx);
        assert!(tab.app.pending_code_exec.is_none());
        assert!(tab.app.messages.iter().any(|m| m.role == "tool"));
    }

    #[test]
    fn handle_code_exec_deny_emits_tool_error() {
        let registry = registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        handle_code_exec_deny(&mut tab, 0, &registry, &args, &tx);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("用户拒绝执行"))
        );
    }

    #[test]
    fn handle_code_exec_approve_without_pending_adds_message() {
        let registry = registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        handle_code_exec_approve(&mut tab, 0, &registry, &args, &tx);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("没有待审批"))
        );
    }
}
