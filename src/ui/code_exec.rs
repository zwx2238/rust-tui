use crate::ui::state::CodeExecLive;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub(crate) fn run_python_in_docker_stream(
    code: &str,
    live: Arc<Mutex<CodeExecLive>>,
) -> Result<(), String> {
    run_docker_stream(code, live)
}

fn run_docker_stream(code: &str, live: Arc<Mutex<CodeExecLive>>) -> Result<(), String> {
    let mut child = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-i")
        .arg("--network=none")
        .arg("--cpus=1")
        .arg("--memory=512m")
        .arg("--pids-limit=128")
        .arg("--read-only")
        .arg("--cap-drop=ALL")
        .arg("--security-opt=no-new-privileges")
        .arg("--tmpfs")
        .arg("/tmp:rw,noexec,nosuid,size=64m")
        .arg("python:3.11-slim")
        .arg("python")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Docker 启动失败：{e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let mut payload = code.to_string();
        if !payload.ends_with('\n') {
            payload.push('\n');
        }
        stdin
            .write_all(payload.as_bytes())
            .map_err(|e| format!("写入容器失败：{e}"))?;
    }

    let mut stdout = child.stdout.take().ok_or_else(|| "无法读取 stdout".to_string())?;
    let mut stderr = child.stderr.take().ok_or_else(|| "无法读取 stderr".to_string())?;

    let live_out = Arc::clone(&live);
    let t_out = std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if let Ok(mut live) = live_out.lock() {
                        live.stdout.push_str(&chunk);
                    }
                }
                Err(_) => break,
            }
        }
    });

    let live_err = Arc::clone(&live);
    let t_err = std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match stderr.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                    if let Ok(mut live) = live_err.lock() {
                        live.stderr.push_str(&chunk);
                    }
                }
                Err(_) => break,
            }
        }
    });

    let status = child.wait().map_err(|e| format!("Docker 执行失败：{e}"))?;
    let _ = t_out.join();
    let _ = t_err.join();

    if let Ok(mut live) = live.lock() {
        live.exit_code = Some(status.code().unwrap_or(-1));
        live.done = true;
    }
    Ok(())
}
