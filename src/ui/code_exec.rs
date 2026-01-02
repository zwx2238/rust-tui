use crate::ui::state::CodeExecLive;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn run_python_in_docker_stream(
    code: &str,
    live: Arc<Mutex<CodeExecLive>>,
    cancel: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), String> {
    run_docker_stream(code, live, cancel)
}

fn run_docker_stream(
    code: &str,
    live: Arc<Mutex<CodeExecLive>>,
    cancel: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), String> {
    let finished = Arc::new(AtomicBool::new(false));
    let run_id = format!(
        "deepchat-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let cidfile = std::env::temp_dir().join(format!("{run_id}.cid"));
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
        .arg("--cidfile")
        .arg(&cidfile)
        .arg("--label")
        .arg(format!("deepchat-run={run_id}"))
        .arg("python:3.11-slim")
        .arg("python")
        .arg("-u")
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

    let killer_live = Arc::clone(&live);
    let killer_cancel = Arc::clone(&cancel);
    let killer_finished = Arc::clone(&finished);
    let killer_cidfile = cidfile.clone();
    let killer_run_id = run_id.clone();
    let client_pid = child.id();
    let killer = std::thread::spawn(move || {
        while !killer_cancel.load(std::sync::atomic::Ordering::Relaxed)
            && !killer_finished.load(std::sync::atomic::Ordering::Relaxed)
        {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        if killer_cancel.load(std::sync::atomic::Ordering::Relaxed) {
            let mut cid = None;
            for _ in 0..60 {
                if let Ok(text) = std::fs::read_to_string(&killer_cidfile) {
                    let text = text.trim();
                    if !text.is_empty() {
                        cid = Some(text.to_string());
                        break;
                    }
                }
                cid = find_container_id(&killer_run_id);
                if cid.is_some() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            if let Some(cid) = cid {
                let mut attempts = 0;
                loop {
                    if stop_container(&cid) {
                        if let Ok(mut live) = killer_live.lock() {
                            live.stderr.push_str("已停止执行\n");
                            live.exit_code = Some(-1);
                            live.done = true;
                            live.finished_at = Some(std::time::Instant::now());
                        }
                        break;
                    }
                    attempts += 1;
                    if attempts % 10 == 0 {
                        if let Ok(mut live) = killer_live.lock() {
                            live.stderr.push_str("停止中...\n");
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            } else {
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(client_pid.to_string())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                if let Ok(mut live) = killer_live.lock() {
                    live.stderr
                        .push_str("已停止执行（未获取容器标识，已终止启动进程）。\n");
                    live.exit_code = Some(-1);
                    live.done = true;
                    live.finished_at = Some(std::time::Instant::now());
                }
            }
        }
    });

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
    finished.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = t_out.join();
    let _ = t_err.join();
    let _ = killer.join();

    if let Ok(mut live) = live.lock() {
        if !live.done {
            live.exit_code = Some(status.code().unwrap_or(-1));
            live.done = true;
            live.finished_at = Some(std::time::Instant::now());
        }
    }
    let _ = std::fs::remove_file(&cidfile);
    Ok(())
}

fn find_container_id(run_id: &str) -> Option<String> {
    let output = Command::new("docker")
        .arg("ps")
        .arg("-q")
        .arg("--filter")
        .arg(format!("label=deepchat-run={run_id}"))
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let id = text.lines().next()?.trim().to_string();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

fn stop_container(cid: &str) -> bool {
    let _ = Command::new("docker")
        .arg("kill")
        .arg("--signal=SIGINT")
        .arg(cid)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if wait_container_stop(cid, 12) {
        return true;
    }
    let _ = Command::new("docker")
        .arg("kill")
        .arg("--signal=SIGTERM")
        .arg(cid)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if wait_container_stop(cid, 12) {
        return true;
    }
    let _ = Command::new("docker")
        .arg("kill")
        .arg("--signal=SIGKILL")
        .arg(cid)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if wait_container_stop(cid, 12) {
        return true;
    }
    let _ = Command::new("docker")
        .arg("stop")
        .arg("-t")
        .arg("1")
        .arg(cid)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    wait_container_stop(cid, 12)
}

fn wait_container_stop(cid: &str, rounds: usize) -> bool {
    for _ in 0..rounds {
        if let Ok(output) = Command::new("docker")
            .arg("inspect")
            .arg("-f")
            .arg("{{.State.Running}}")
            .arg(cid)
            .output()
        {
            if !output.status.success() {
                return true;
            }
            let text = String::from_utf8_lossy(&output.stdout);
            if text.trim() == "false" {
                return true;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    false
}
