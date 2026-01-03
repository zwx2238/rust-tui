#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::types::{ToolCall, ToolFunctionCall};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::tool_service::ToolService;
    use std::sync::mpsc;

    fn registry_empty_key() -> ModelRegistry {
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

    fn args(enable: Option<String>, yolo: bool) -> Args {
        Args {
            model: "m".to_string(),
            system: "sys".to_string(),
            base_url: "http://example.com".to_string(),
            show_reasoning: false,
            config: None,
            resume: None,
            replay_fork_last: false,
            enable,
            log_requests: None,
            perf: false,
            question_set: None,
            yolo,
            read_only: false,
            wait_gdb: false,
        }
    }

    fn tool_call(name: &str, args: &str) -> ToolCall {
        ToolCall {
            id: "call1".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: name.to_string(),
                arguments: args.to_string(),
            },
        }
    }

    #[test]
    fn web_search_disabled_adds_error_message() {
        let registry = registry_empty_key();
        let args = args(None, false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call("web_search", r#"{"query":"hi"}"#)];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("web_search 未启用"))
        );
    }

    #[test]
    fn code_exec_enabled_sets_pending_request() {
        let registry = registry_empty_key();
        let args = args(Some("code_exec".to_string()), false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call(
            "code_exec",
            r#"{"language":"python","code":"print(1)"}"#,
        )];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(tab.app.pending_code_exec.is_some());
    }

    #[test]
    fn read_file_enabled_reads_file() {
        let registry = registry_empty_key();
        let args = args(None, false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let path = std::env::temp_dir().join("deepchat_read_file.txt");
        std::fs::write(&path, "hello").unwrap();
        let calls = vec![tool_call(
            "read_file",
            &format!(r#"{{"path":"{}"}}"#, path.to_string_lossy()),
        )];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("[read_file]"))
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn modify_file_blocked_in_read_only_mode() {
        let registry = registry_empty_key();
        let mut args = args(None, false);
        args.read_only = true;
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call(
            "modify_file",
            r#"{"diff":"diff --git a/a b/a\n","path":"a"}"#,
        )];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("read_only"))
        );
    }

    #[test]
    fn web_search_enabled_reports_missing_key() {
        let registry = registry_empty_key();
        let args = args(Some("web_search".to_string()), false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call("web_search", r#"{"query":"hi"}"#)];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("tavily_api_key"))
        );
    }

    #[test]
    fn read_code_enabled_reads_file_with_numbers() {
        let registry = registry_empty_key();
        let args = args(None, false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let path = std::env::temp_dir().join("deepchat_read_code.rs");
        std::fs::write(&path, "line1\nline2").unwrap();
        let calls = vec![tool_call(
            "read_code",
            &format!(r#"{{"path":"{}"}}"#, path.to_string_lossy()),
        )];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("1 | line1"))
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn code_exec_disabled_adds_error_message() {
        let registry = registry_empty_key();
        let args = args(Some("-code_exec".to_string()), false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call(
            "code_exec",
            r#"{"language":"python","code":"print(1)"}"#,
        )];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("code_exec 未启用"))
        );
    }

    #[test]
    fn read_file_disabled_adds_error_message() {
        let registry = registry_empty_key();
        let args = args(Some("-read_file".to_string()), false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call("read_file", r#"{"path":"a.txt"}"#)];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("read_file 未启用"))
        );
    }

    #[test]
    fn modify_file_invalid_json_reports_error() {
        let registry = registry_empty_key();
        let args = args(Some("modify_file".to_string()), false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call("modify_file", "{")];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("modify_file 参数解析失败"))
        );
    }

    #[test]
    fn empty_tool_calls_adds_fallback_message() {
        let registry = registry_empty_key();
        let args = args(None, false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        service.apply_tool_calls(&mut tab, 0, &[]);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("未找到可靠结果"))
        );
    }
}
