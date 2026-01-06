use crate::args::Args;
use crate::model_registry::{ModelProfile, ModelRegistry};
use crate::test_support::{restore_env, set_env};
use crate::types::{ToolCall, ToolFunctionCall};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const DOCKER_SCRIPT: &str = r#"#!/bin/sh
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
        input="$(printf "%s" "$input" | python -c "import json, os, sys
ws = os.environ.get('DEEPCHAT_WORKSPACE')
data = json.load(sys.stdin)
if ws and isinstance(data, dict) and isinstance(data.get('path'), str):
    if data['path'].startswith('/workspace'):
        suffix = data['path'][len('/workspace'):].lstrip('/')
        data['path'] = os.path.join(ws, suffix)
print(json.dumps(data, ensure_ascii=False))")"
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

pub(super) struct FakeDocker {
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

pub(super) fn setup_fake_docker(workspace: &str) -> FakeDocker {
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

pub(super) fn registry_empty_key() -> ModelRegistry {
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

pub(super) fn args(enable: Option<String>, yolo: bool) -> Args {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("deepchat-tool-workspace-{id}"));
    let _ = std::fs::create_dir_all(&dir);
    let workspace = dir.to_string_lossy().to_string();
    Args {
        model: "m".to_string(),
        system: "sys".to_string(),
        base_url: "http://example.com".to_string(),
        show_reasoning: false,
        resume: None,
        replay_fork_last: false,
        enable,
        log_requests: None,
        perf: false,
        question_set: None,
        workspace,
        yolo,
        read_only: false,
        wait_gdb: false,
    }
}

pub(super) fn tool_call(name: &str, args: &str) -> ToolCall {
    ToolCall {
        id: "call1".to_string(),
        kind: "function".to_string(),
        function: ToolFunctionCall {
            name: name.to_string(),
            arguments: args.to_string(),
        },
    }
}
