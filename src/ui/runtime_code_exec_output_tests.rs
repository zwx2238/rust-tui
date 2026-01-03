#[cfg(test)]
mod tests {
    use crate::ui::runtime_code_exec_output::{
        build_code_exec_tool_output, escape_json_string, take_code_exec_reason,
    };
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::state::{CodeExecLive, CodeExecReasonTarget, PendingCodeExec};
    use std::time::Instant;

    #[test]
    fn build_tool_output_contains_sections() {
        let pending = PendingCodeExec {
            call_id: "c".to_string(),
            language: "python".to_string(),
            code: "print(1)".to_string(),
            exec_code: None,
            requested_at: Instant::now(),
            stop_reason: Some("stop".to_string()),
        };
        let live = CodeExecLive {
            started_at: Instant::now(),
            finished_at: Some(Instant::now()),
            stdout: "ok".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            done: true,
        };
        let out = build_code_exec_tool_output(&pending, &live);
        assert!(out.contains("stdout:"));
        assert!(out.contains("stop_reason"));
    }

    #[test]
    fn take_reason_clears_input() {
        let mut tab = TabState::new("id".into(), "cat".into(), "", false, "m", "p");
        tab.app.code_exec_reason_target = Some(CodeExecReasonTarget::Stop);
        tab.app.code_exec_reason_input.insert_str("原因");
        let reason = take_code_exec_reason(&mut tab, CodeExecReasonTarget::Stop);
        assert_eq!(reason.as_deref(), Some("原因"));
        assert_eq!(tab.app.code_exec_reason_input.lines().len(), 1);
        assert!(
            tab.app
                .code_exec_reason_input
                .lines()
                .first()
                .map(|l| l.is_empty())
                .unwrap_or(true)
        );
    }

    #[test]
    fn escape_json_string_replaces_chars() {
        let input = "a\"b\\c\nd";
        let out = escape_json_string(input);
        assert!(out.contains("\\\""));
        assert!(out.contains("\\\\"));
        assert!(out.contains("\\n"));
    }
}
