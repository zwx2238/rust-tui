use crate::ui::workspace::WorkspaceConfig;
use crate::test_support::{restore_env, set_env};
use std::fs;
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

pub(super) fn temp_dir(name: &str) -> PathBuf {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("deepchat-{name}-{id}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub(super) fn workspace() -> WorkspaceConfig {
    let dir = temp_dir("workspace");
    WorkspaceConfig {
        host_path: dir,
        mount_path: "/workspace".to_string(),
    }
}

pub(super) struct FakeDocker {
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

pub(super) fn setup_fake_docker(workspace: &WorkspaceConfig) -> FakeDocker {
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
