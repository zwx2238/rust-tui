#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::types::{ToolCall, ToolFunctionCall};
    use crate::ui::runtime_file_patch::{handle_file_patch_cancel, handle_file_patch_request};
    use crate::ui::runtime_helpers::TabState;
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

    fn patch_call(args: &str) -> ToolCall {
        ToolCall {
            id: "call1".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "modify_file".to_string(),
                arguments: args.to_string(),
            },
        }
    }

    #[test]
    fn handle_file_patch_request_sets_pending() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let call = patch_call(r#"{"diff":"diff --git a/a b/a\n","path":"a"}"#);
        handle_file_patch_request(&mut tab, &call).unwrap();
        assert!(tab.app.pending_file_patch.is_some());
    }

    #[test]
    fn handle_file_patch_request_rejects_empty_diff() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let call = patch_call(r#"{"diff":"   "}"#);
        let err = handle_file_patch_request(&mut tab, &call).unwrap_err();
        assert!(err.contains("diff 不能为空"));
    }

    #[test]
    fn handle_file_patch_cancel_emits_tool_error() {
        let registry = registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let call = patch_call(r#"{"diff":"diff --git a/a b/a\n","path":"a"}"#);
        handle_file_patch_request(&mut tab, &call).unwrap();
        handle_file_patch_cancel(&mut tab, 0, &registry, &args, &tx);
        assert!(tab.app.messages.iter().any(|m| m.content.contains("用户取消")));
    }

    #[test]
    fn handle_file_patch_request_rejects_invalid_json() {
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let call = patch_call("{");
        let err = handle_file_patch_request(&mut tab, &call).unwrap_err();
        assert!(err.contains("参数解析失败"));
    }

    #[test]
    fn handle_file_patch_apply_reports_error() {
        let registry = registry();
        let args = args();
        let (tx, _rx) = mpsc::channel();
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        tab.app.pending_file_patch = Some(crate::ui::state::PendingFilePatch {
            call_id: "call".to_string(),
            path: Some("nope".to_string()),
            diff: "diff --git a/nope b/nope\nnew file mode 100644\nindex 0000000..0000000\n--- /dev/null\n+++ b/nope\n@@\n+hi\n".to_string(),
            preview: "preview".to_string(),
        });
        crate::ui::runtime_file_patch::handle_file_patch_apply(&mut tab, 0, &registry, &args, &tx);
        assert!(tab
            .app
            .messages
            .iter()
            .any(|m| m.role == crate::types::ROLE_TOOL));
    }
}
