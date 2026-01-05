#[cfg(test)]
mod tests {
    use crate::types::{ToolCall, ToolFunctionCall};
    use crate::ui::tools::{parse_code_exec_args, run_tool};
    use crate::ui::workspace::WorkspaceConfig;
    use crate::test_support::{env_lock, restore_env, set_env};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn temp_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("deepchat-{name}-{id}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn workspace() -> WorkspaceConfig {
        let dir = temp_dir("workspace");
        WorkspaceConfig {
            host_path: dir,
            mount_path: "/workspace".to_string(),
        }
    }

    struct FakeDocker {
        dir: PathBuf,
        prev_path: Option<String>,
        prev_workspace: Option<String>,
    }

    impl Drop for FakeDocker {
        fn drop(&mut self) {
            restore_env("PATH", self.prev_path.take());
            restore_env("DEEPCHAT_WORKSPACE", self.prev_workspace.take());
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn setup_fake_docker(workspace: &WorkspaceConfig) -> FakeDocker {
        let dir = temp_dir("fake-docker");
        let bin = dir.join("docker");
        fs::write(&bin, DOCKER_SCRIPT).unwrap();
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
        let prev_workspace = set_env(
            "DEEPCHAT_WORKSPACE",
            &workspace.host_path.to_string_lossy(),
        );
        FakeDocker {
            dir,
            prev_path,
            prev_workspace,
        }
    }

    #[test]
    fn parse_code_exec_rejects_empty() {
        assert!(parse_code_exec_args(r#"{"language":"","code":""}"#).is_err());
    }

    #[test]
    fn parse_code_exec_rejects_invalid_json() {
        let err = parse_code_exec_args("{").err().unwrap();
        assert!(err.contains("参数解析失败"));
    }

    #[test]
    fn parse_code_exec_rejects_non_python() {
        let err = parse_code_exec_args(r#"{"language":"js","code":"1"}"#)
            .err()
            .unwrap();
        assert!(err.contains("仅支持"));
    }

    #[test]
    fn parse_code_exec_rejects_empty_code() {
        let err = parse_code_exec_args(r#"{"language":"python","code":"  "}"#)
            .err()
            .unwrap();
        assert!(err.contains("code 不能为空"));
    }

    #[test]
    fn parse_code_exec_accepts_python() {
        let req = parse_code_exec_args(r#"{"language":"python","code":"print(1)"}"#).unwrap();
        assert_eq!(req.language, "python");
        assert!(req.code.contains("print"));
    }

    #[test]
    fn read_file_respects_root() {
        let _guard = env_lock().lock().unwrap();
        let ws = workspace();
        let _docker = setup_fake_docker(&ws);
        let root = ws.host_path.clone();
        let good_path = root.join("a.txt");
        fs::write(&good_path, "hello").unwrap();
        let call = ToolCall {
            id: "1".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, good_path.display()),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("[read_file]"));
        assert!(result.content.contains("hello"));
        let bad_path = temp_dir("tools-outside").join("b.txt");
        fs::write(&bad_path, "nope").unwrap();
        let call = ToolCall {
            id: "2".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, bad_path.display()),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("workspace 之外"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn read_file_empty_path_errors() {
        let call = ToolCall {
            id: "3".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: r#"{"path":""}"#.to_string(),
            },
        };
        let result = run_tool(&call, "", &workspace());
        assert!(result.content.contains("path 不能为空"));
    }

    #[test]
    fn read_file_invalid_json_errors() {
        let call = ToolCall {
            id: "3b".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: "{".to_string(),
            },
        };
        let result = run_tool(&call, "", &workspace());
        assert!(result.content.contains("参数解析失败"));
    }

    #[test]
    fn read_file_too_large_errors() {
        let _guard = env_lock().lock().unwrap();
        let ws = workspace();
        let _docker = setup_fake_docker(&ws);
        let dir = ws.host_path.clone();
        let path = dir.join("big.txt");
        let data = "a".repeat(1024);
        fs::write(&path, data).unwrap();
        let call = ToolCall {
            id: "3c".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(r#"{{"path":"{}","max_bytes":1}}"#, path.display()),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("文件过大"));
        let _ = fs::remove_dir_all(&ws.host_path);
    }

    #[test]
    fn read_file_respects_line_range() {
        let _guard = env_lock().lock().unwrap();
        let ws = workspace();
        let _docker = setup_fake_docker(&ws);
        let dir = ws.host_path.clone();
        let path = dir.join("lines.txt");
        fs::write(&path, "a\nb\nc").unwrap();
        let call = ToolCall {
            id: "3d".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_file".to_string(),
                arguments: format!(
                    r#"{{"path":"{}","start_line":2,"end_line":2}}"#,
                    path.display()
                ),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("b"));
        let _ = fs::remove_dir_all(&ws.host_path);
    }

    #[test]
    fn read_code_includes_line_numbers() {
        let _guard = env_lock().lock().unwrap();
        let ws = workspace();
        let _docker = setup_fake_docker(&ws);
        let dir = ws.host_path.clone();
        let path = dir.join("a.rs");
        fs::write(&path, "line1\nline2").unwrap();
        let call = ToolCall {
            id: "4".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "read_code".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, path.display()),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("1 | line1"));
        let _ = fs::remove_dir_all(&ws.host_path);
    }

    #[test]
    fn list_dir_outputs_entries() {
        let _guard = env_lock().lock().unwrap();
        let ws = workspace();
        let _docker = setup_fake_docker(&ws);
        let dir = ws.host_path.clone();
        let file = dir.join("a.txt");
        let sub = dir.join("sub");
        fs::write(&file, "hi").unwrap();
        fs::create_dir_all(&sub).unwrap();
        let call = ToolCall {
            id: "8".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "list_dir".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, dir.display()),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("a.txt"));
        let _ = fs::remove_dir_all(&ws.host_path);
    }

    #[test]
    fn list_dir_respects_root() {
        let _guard = env_lock().lock().unwrap();
        let ws = workspace();
        let _docker = setup_fake_docker(&ws);
        let root = ws.host_path.clone();
        let good = root.join("ok");
        fs::create_dir_all(&good).unwrap();
        let call = ToolCall {
            id: "9".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "list_dir".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, root.display()),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("[list_dir]"));
        let outside = temp_dir("tools-list-outside");
        let call = ToolCall {
            id: "10".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "list_dir".to_string(),
                arguments: format!(r#"{{"path":"{}"}}"#, outside.display()),
            },
        };
        let result = run_tool(&call, "", &ws);
        assert!(result.content.contains("workspace 之外"));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
    }

    #[test]
    fn web_search_requires_query_and_key() {
        let call = ToolCall {
            id: "5".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "web_search".to_string(),
                arguments: r#"{"query":""}"#.to_string(),
            },
        };
        let result = run_tool(&call, "", &workspace());
        assert!(result.content.contains("query 不能为空"));

        let call = ToolCall {
            id: "6".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "web_search".to_string(),
                arguments: r#"{"query":"hi"}"#.to_string(),
            },
        };
        let result = run_tool(&call, "", &workspace());
        assert!(result.content.contains("tavily_api_key"));
    }

    #[test]
    fn web_search_invalid_args_reports_error() {
        let call = ToolCall {
            id: "6b".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "web_search".to_string(),
                arguments: "{".to_string(),
            },
        };
        let result = run_tool(&call, "", &workspace());
        assert!(result.content.contains("参数解析失败"));
    }

    #[test]
    fn unknown_tool_returns_message() {
        let call = ToolCall {
            id: "7".to_string(),
            kind: "function".to_string(),
            function: ToolFunctionCall {
                name: "unknown".to_string(),
                arguments: "{}".to_string(),
            },
        };
        let result = run_tool(&call, "", &workspace());
        assert!(result.content.contains("未知工具"));
    }
}
