use crate::ui::state::CodeExecLive;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, OnceLock};

use crate::ui::code_exec_container_env::{pip_target_dir, run_dir, tmp_dir};
use crate::ui::workspace::WorkspaceConfig;

mod container_start;
use container_start::{is_container_running, start_container};

pub(crate) fn ensure_container(
    container_id: &mut Option<String>,
    workspace: &WorkspaceConfig,
) -> Result<String, String> {
    if let Some(id) = container_id.as_ref()
        && is_container_running(id)
    {
        return Ok(id.clone());
    }
    let id = start_container(Some(workspace))?;
    *container_id = Some(id.clone());
    Ok(id)
}

static CONTAINER_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn cached_container() -> &'static Mutex<Option<String>> {
    CONTAINER_CACHE.get_or_init(|| Mutex::new(None))
}

pub(crate) fn ensure_container_cached(
    workspace: &WorkspaceConfig,
) -> Result<String, String> {
    let mut guard = cached_container()
        .lock()
        .map_err(|_| "Docker 启动失败：容器缓存锁异常".to_string())?;
    if let Some(id) = guard.as_ref()
        && is_container_running(id)
    {
        return Ok(id.clone());
    }
    let id = start_container(Some(workspace))?;
    *guard = Some(id.clone());
    Ok(id)
}

pub(crate) fn run_python_in_container_stream(
    container_id: &str,
    run_id: &str,
    code: &str,
    live: Arc<Mutex<CodeExecLive>>,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    let finished = Arc::new(AtomicBool::new(false));
    write_code_file(container_id, run_id, code)?;
    let mut child = spawn_python_exec(container_id, run_id)?;
    let (stdout, stderr) = take_child_pipes(&mut child)?;
    let t_out = spawn_stream_reader(stdout, Arc::clone(&live), OutputTarget::Stdout);
    let t_err = spawn_stream_reader(stderr, Arc::clone(&live), OutputTarget::Stderr);
    let killer = spawn_cancel_watcher(container_id, run_id, &live, &cancel, &finished);
    let status = child.wait().map_err(|e| format!("Docker 执行失败：{e}"))?;
    finalize_exec(status.code(), &live, &finished, t_out, t_err, killer);
    let _ = remove_code_file(container_id, run_id);
    Ok(())
}

pub(crate) fn run_bash_in_container_stream(
    container_id: &str,
    run_id: &str,
    code: &str,
    live: Arc<Mutex<CodeExecLive>>,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    let finished = Arc::new(AtomicBool::new(false));
    write_script_file(container_id, run_id, "sh", code)?;
    let mut child = spawn_bash_exec(container_id, run_id)?;
    let (stdout, stderr) = take_child_pipes(&mut child)?;
    let t_out = spawn_stream_reader(stdout, Arc::clone(&live), OutputTarget::Stdout);
    let t_err = spawn_stream_reader(stderr, Arc::clone(&live), OutputTarget::Stderr);
    let killer = spawn_cancel_watcher(container_id, run_id, &live, &cancel, &finished);
    let status = child.wait().map_err(|e| format!("Docker 执行失败：{e}"))?;
    finalize_exec(status.code(), &live, &finished, t_out, t_err, killer);
    let _ = remove_script_file(container_id, run_id, "sh");
    Ok(())
}

enum OutputTarget {
    Stdout,
    Stderr,
}

fn spawn_python_exec(container_id: &str, run_id: &str) -> Result<std::process::Child, String> {
    let mut cmd = Command::new("docker");
    cmd.arg("exec")
        .arg("-i")
        .arg(container_id)
        .arg("python")
        .arg("-u")
        .arg(code_path(run_id))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.spawn().map_err(|e| format!("Docker 执行失败：{e}"))
}

fn spawn_bash_exec(container_id: &str, run_id: &str) -> Result<std::process::Child, String> {
    let mut cmd = Command::new("docker");
    cmd.arg("exec")
        .arg("-i")
        .arg(container_id)
        .arg("bash")
        .arg(script_path(run_id, "sh"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.spawn().map_err(|e| format!("Docker 执行失败：{e}"))
}

fn take_child_pipes(
    child: &mut std::process::Child,
) -> Result<(std::process::ChildStdout, std::process::ChildStderr), String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取 stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法读取 stderr".to_string())?;
    Ok((stdout, stderr))
}

fn spawn_stream_reader(
    mut stream: impl Read + Send + 'static,
    live: Arc<Mutex<CodeExecLive>>,
    target: OutputTarget,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => append_live_output(&live, &buf[..n], &target),
                Err(_) => break,
            }
        }
    })
}

fn append_live_output(live: &Arc<Mutex<CodeExecLive>>, data: &[u8], target: &OutputTarget) {
    let chunk = String::from_utf8_lossy(data).to_string();
    if let Ok(mut live) = live.lock() {
        match target {
            OutputTarget::Stdout => live.stdout.push_str(&chunk),
            OutputTarget::Stderr => live.stderr.push_str(&chunk),
        }
    }
}

fn spawn_cancel_watcher(
    container_id: &str,
    run_id: &str,
    live: &Arc<Mutex<CodeExecLive>>,
    cancel: &Arc<AtomicBool>,
    finished: &Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    let live_kill = Arc::clone(live);
    let cancel_kill = Arc::clone(cancel);
    let finished_kill = Arc::clone(finished);
    let cid = container_id.to_string();
    let run_id_kill = run_id.to_string();
    std::thread::spawn(move || {
        while !cancel_kill.load(std::sync::atomic::Ordering::Relaxed)
            && !finished_kill.load(std::sync::atomic::Ordering::Relaxed)
        {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        if cancel_kill.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = stop_exec(&cid, &run_id_kill);
            mark_cancelled(&live_kill);
        }
    })
}

fn mark_cancelled(live: &Arc<Mutex<CodeExecLive>>) {
    if let Ok(mut live) = live.lock() {
        live.stderr.push_str("已停止执行\n");
        live.exit_code = Some(-1);
        live.done = true;
        live.finished_at = Some(std::time::Instant::now());
    }
}

fn finalize_exec(
    status_code: Option<i32>,
    live: &Arc<Mutex<CodeExecLive>>,
    finished: &Arc<AtomicBool>,
    t_out: std::thread::JoinHandle<()>,
    t_err: std::thread::JoinHandle<()>,
    killer: std::thread::JoinHandle<()>,
) {
    finished.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = t_out.join();
    let _ = t_err.join();
    let _ = killer.join();
    if let Ok(mut live) = live.lock()
        && !live.done
    {
        live.exit_code = Some(status_code.unwrap_or(-1));
        live.done = true;
        live.finished_at = Some(std::time::Instant::now());
    }
}

pub(crate) fn stop_exec(container_id: &str, run_id: &str) -> bool {
    let _ = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("pkill -f {}/{}.", run_dir(), run_id))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    true
}

fn write_code_file(container_id: &str, run_id: &str, code: &str) -> Result<(), String> {
    let run_dir = run_dir();
    let tmp_dir = tmp_dir();
    let site_dir = pip_target_dir();
    ensure_container_dirs(container_id, &run_dir, &tmp_dir, &site_dir);
    write_code_via_stdin(container_id, &code_path(run_id), code)
}

fn ensure_container_dirs(container_id: &str, run_dir: &str, tmp_dir: &str, site_dir: &str) {
    let _ = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("mkdir -p {run_dir} {tmp_dir} {site_dir}"))
        .status();
}

fn write_code_via_stdin(container_id: &str, path: &str, code: &str) -> Result<(), String> {
    let mut cmd = Command::new("docker");
    cmd.arg("exec")
        .arg("-i")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("cat > {}", path))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = cmd.spawn().map_err(|e| format!("写入容器失败：{e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        let mut payload = code.to_string();
        if !payload.ends_with('\n') {
            payload.push('\n');
        }
        stdin
            .write_all(payload.as_bytes())
            .map_err(|e| format!("写入容器失败：{e}"))?;
    }
    let status = child.wait().map_err(|e| format!("写入容器失败：{e}"))?;
    if !status.success() {
        return Err("写入容器失败".to_string());
    }
    Ok(())
}

fn remove_code_file(container_id: &str, run_id: &str) -> Result<(), String> {
    let _ = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("rm -f {}", code_path(run_id)))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Ok(())
}

fn write_script_file(
    container_id: &str,
    run_id: &str,
    ext: &str,
    code: &str,
) -> Result<(), String> {
    let run_dir = run_dir();
    let tmp_dir = tmp_dir();
    let site_dir = pip_target_dir();
    ensure_container_dirs(container_id, &run_dir, &tmp_dir, &site_dir);
    write_code_via_stdin(container_id, &script_path(run_id, ext), code)
}

fn remove_script_file(container_id: &str, run_id: &str, ext: &str) -> Result<(), String> {
    let _ = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("rm -f {}", script_path(run_id, ext)))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Ok(())
}

fn code_path(run_id: &str) -> String {
    script_path(run_id, "py")
}

fn script_path(run_id: &str, ext: &str) -> String {
    format!("{}/{}.{}", run_dir(), run_id, ext)
}
