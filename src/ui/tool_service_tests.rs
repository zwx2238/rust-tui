#[cfg(test)]
mod tests {
    use crate::args::Args;
    use crate::model_registry::{ModelProfile, ModelRegistry};
    use crate::types::{ToolCall, ToolFunctionCall};
    use crate::ui::runtime_helpers::TabState;
    use crate::ui::tool_service::ToolService;
    use crate::test_support::{env_lock, restore_env, set_env};
    use std::path::PathBuf;
    use std::sync::mpsc;

    const DOCKER_SCRIPT: &str = r#"#!/bin/sh
cmd="$1"
shift
case "$cmd" in
  inspect)
    echo "true"
    exit 0
    ;;
  run)
    echo "dummy-container"
    exit 0
    ;;
  exec)
    if [ "$1" = "-i" ]; then shift; fi
    shift
    if [ "$1" = "python" ] && [ "$2" = "-c" ]; then
      script="$3"
      input="$(cat)"
      if [ -n "$DEEPCHAT_WORKSPACE" ]; then
        input="$(printf "%s" "$input" | python - <<'PY'
import json, os, sys
ws = os.environ.get("DEEPCHAT_WORKSPACE")
data = json.load(sys.stdin)
if ws and isinstance(data, dict) and isinstance(data.get("path"), str):
    if data["path"].startswith("/workspace"):
        suffix = data["path"][len("/workspace"):].lstrip("/")
        data["path"] = os.path.join(ws, suffix)
print(json.dumps(data, ensure_ascii=False))
PY
)"
      fi
      printf "%s" "$input" | python -c "$script"
      exit $?
    fi
    cat >/dev/null
    exit 0
    ;;
  *)
    exit 0
    ;;
esac
"#;

    struct FakeDocker {
        dir: PathBuf,
        prev_path: Option<String>,
        prev_workspace: Option<String>,
    }

    impl Drop for FakeDocker {
        fn drop(&mut self) {
            restore_env("PATH", self.prev_path.take());
            restore_env("DEEPCHAT_WORKSPACE", self.prev_workspace.take());
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn setup_fake_docker(workspace: &str) -> FakeDocker {
        let dir = std::env::temp_dir().join(format!(
            "deepchat-docker-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("docker");
        std::fs::write(&bin, DOCKER_SCRIPT).unwrap();
        let _ = std::process::Command::new("chmod")
            .arg("+x")
            .arg(&bin)
            .status();
        let prev_path = set_env(
            "PATH",
            &format!(
                "{}:{}",
                dir.to_string_lossy(),
                std::env::var("PATH").unwrap_or_default()
            ),
        );
        let prev_workspace = set_env("DEEPCHAT_WORKSPACE", workspace);
        FakeDocker {
            dir,
            prev_path,
            prev_workspace,
        }
    }

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
            workspace: "/tmp/deepchat-workspace".to_string(),
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
        let _guard = env_lock().lock().unwrap();
        let registry = registry_empty_key();
        let args = args(None, false);
        let _ = std::fs::create_dir_all(&args.workspace);
        let _docker = setup_fake_docker(&args.workspace);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let path = PathBuf::from(&args.workspace).join("deepchat_read_file.txt");
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
                .any(|m| m.content.contains("hello"))
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
        let _guard = env_lock().lock().unwrap();
        let registry = registry_empty_key();
        let args = args(None, false);
        let _ = std::fs::create_dir_all(&args.workspace);
        let _docker = setup_fake_docker(&args.workspace);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let path = PathBuf::from(&args.workspace).join("deepchat_read_code.rs");
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
    fn list_dir_disabled_when_read_file_disabled() {
        let registry = registry_empty_key();
        let args = args(Some("-read_file".to_string()), false);
        let (tx, _rx) = mpsc::channel();
        let service = ToolService::new(&registry, &args, &tx);
        let mut tab = TabState::new("id".into(), "默认".into(), "", false, "m1", "p1");
        let calls = vec![tool_call("list_dir", r#"{"path":"."}"#)];
        service.apply_tool_calls(&mut tab, 0, &calls);
        assert!(
            tab.app
                .messages
                .iter()
                .any(|m| m.content.contains("list_dir 未启用"))
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
