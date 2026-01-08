#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::runtime_yolo::auto_finalize_code_exec;
    use std::sync::mpsc;

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

    fn args(yolo: bool) -> Args {
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
            yolo,
            read_only: false,
            wait_gdb: false,
        }
    }

    #[test]
    fn auto_finalize_skips_when_disabled() {
        let registry = registry();
        let args = args(false);
        let (tx, _rx) = mpsc::channel();
        let mut tabs = vec![TabState::new(
            "id".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        tabs[0].app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        tabs[0].app.code_exec_result_ready = true;
        tabs[0].app.code_exec_finished_output = Some("output".to_string());
        auto_finalize_code_exec(&mut tabs, &registry, &args, &tx);
        assert!(tabs[0].app.pending_code_exec.is_some());
    }

    #[test]
    fn auto_finalize_exits_when_ready() {
        let registry = registry();
        let args = args(true);
        let (tx, _rx) = mpsc::channel();
        let mut tabs = vec![TabState::new(
            "id".into(),
            "默认".into(),
            "",
            false,
            "m1",
            "p1",
        )];
        tabs[0].app.pending_code_exec = Some(crate::ui::state::PendingCodeExec {
            call_id: "call".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: std::time::Instant::now(),
            stop_reason: None,
        });
        tabs[0].app.code_exec_result_ready = true;
        tabs[0].app.code_exec_finished_output = Some("output".to_string());
        auto_finalize_code_exec(&mut tabs, &registry, &args, &tx);
        assert!(tabs[0].app.pending_code_exec.is_none());
        assert!(tabs[0].app.messages.iter().any(|m| m.role == "tool"));
    }
}
