#[cfg(test)]
mod tests {
    use crate::test_support::{env_lock, restore_env, set_env};
    use crate::ui::code_exec_container::{
        ensure_container, run_python_in_container_stream, stop_exec,
    };
    use crate::ui::state::CodeExecLive;
    use std::fs;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::ui::workspace::WorkspaceConfig;

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
    if echo "$*" | grep -q "cat >"; then
      cat >/dev/null
      exit 0
    fi
    for arg in "$@"; do
      if [ "$arg" = "python" ]; then
        echo "ok"
        exit 0
      fi
    done
    exit 0
    ;;
  *)
    exit 0
    ;;
esac
"#;

    fn docker_script() -> &'static str {
        DOCKER_SCRIPT
    }

    fn write_executable(bin: &std::path::Path, script: &str) {
        fs::write(bin, script).unwrap();
        let _ = std::process::Command::new("chmod")
            .arg("+x")
            .arg(bin)
            .status();
    }

    fn set_fake_path(dir: &std::path::Path) -> Option<String> {
        set_env(
            "PATH",
            &format!(
                "{}:{}",
                dir.to_string_lossy(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
    }

    fn setup_fake_docker() -> (std::path::PathBuf, Option<String>) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("deepchat-docker-{ts}"));
        fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("docker");
        write_executable(&bin, docker_script());
        let prev_path = set_fake_path(&dir);
        (dir, prev_path)
    }

    fn workspace() -> WorkspaceConfig {
        let dir = std::env::temp_dir().join("deepchat-workspace-test");
        let _ = fs::create_dir_all(&dir);
        WorkspaceConfig {
            host_path: dir,
            mount_path: "/workspace".to_string(),
        }
    }

    #[test]
    fn ensure_container_uses_fake_docker() {
        let _guard = env_lock().lock().unwrap();
        let (dir, prev_path) = setup_fake_docker();
        let mut container_id = None;
        let ws = workspace();
        let id = ensure_container(&mut container_id, &ws).unwrap();
        assert_eq!(id, "dummy-container");
        restore_env("PATH", prev_path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_python_stream_reads_output() {
        let _guard = env_lock().lock().unwrap();
        let (dir, prev_path) = setup_fake_docker();
        let live = Arc::new(Mutex::new(CodeExecLive {
            started_at: std::time::Instant::now(),
            finished_at: None,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            done: false,
        }));
        let cancel = Arc::new(AtomicBool::new(false));
        let res = run_python_in_container_stream(
            "dummy-container",
            "run-1",
            "print('hi')",
            live.clone(),
            cancel,
        );
        assert!(res.is_ok());
        let out = live.lock().unwrap().stdout.clone();
        assert!(out.contains("ok"));
        restore_env("PATH", prev_path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn stop_exec_returns_true() {
        let _guard = env_lock().lock().unwrap();
        let (dir, prev_path) = setup_fake_docker();
        assert!(stop_exec("dummy", "run-1"));
        restore_env("PATH", prev_path);
        let _ = fs::remove_dir_all(&dir);
    }
}
