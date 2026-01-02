use crate::ui::state::CodeExecLive;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ui::code_exec_container_env::{
    code_exec_network_mode, pip_cache_dir, pip_extra_index_url, pip_index_url, pip_target_dir,
    prepare_pip_cache_dir, run_dir, site_tmpfs_mb, tmp_dir, work_dir,
};

pub(crate) fn ensure_container(container_id: &mut Option<String>) -> Result<String, String> {
    if let Some(id) = container_id.as_ref() {
        if is_container_running(id) {
            return Ok(id.clone());
        }
    }
    let id = start_container()?;
    *container_id = Some(id.clone());
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

    let mut child = cmd.spawn().map_err(|e| format!("Docker 执行失败：{e}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法读取 stdout".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法读取 stderr".to_string())?;

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

    let live_kill = Arc::clone(&live);
    let cancel_kill = Arc::clone(&cancel);
    let finished_kill = Arc::clone(&finished);
    let cid = container_id.to_string();
    let run_id = run_id.to_string();
    let run_id_kill = run_id.clone();
    let killer = std::thread::spawn(move || {
        while !cancel_kill.load(std::sync::atomic::Ordering::Relaxed)
            && !finished_kill.load(std::sync::atomic::Ordering::Relaxed)
        {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        if cancel_kill.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = stop_exec(&cid, &run_id_kill);
            if let Ok(mut live) = live_kill.lock() {
                live.stderr.push_str("已停止执行\n");
                live.exit_code = Some(-1);
                live.done = true;
                live.finished_at = Some(std::time::Instant::now());
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
    let _ = remove_code_file(container_id, &run_id);
    Ok(())
}

pub(crate) fn stop_exec(container_id: &str, run_id: &str) -> bool {
    let _ = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("pkill -f {}", code_path(run_id)))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    true
}

fn start_container() -> Result<String, String> {
    prepare_pip_cache_dir();
    let run_id = format!(
        "deepchat-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let mut cmd = Command::new("docker");
    let work_dir = work_dir();
    let tmp_dir = tmp_dir();
    let site_dir = pip_target_dir();
    cmd.arg("run")
        .arg("-d")
        .arg("--cpus=1")
        .arg("--memory=512m")
        .arg("--pids-limit=128")
        .arg("--read-only")
        .arg("--user=1000:1000")
        .arg("--cap-drop=ALL")
        .arg("--security-opt=no-new-privileges")
        .arg("--tmpfs")
        .arg(format!(
            "{work_dir}:rw,exec,nosuid,size={}m,uid=1000,gid=1000,mode=755",
            site_tmpfs_mb()
        ))
        .arg("-e")
        .arg(format!("TMPDIR={tmp_dir}"))
        .arg("-e")
        .arg(format!("TMP={tmp_dir}"))
        .arg("-e")
        .arg(format!("TEMP={tmp_dir}"))
        .arg("-e")
        .arg(format!("HOME={work_dir}"))
        .arg("-e")
        .arg(format!("DEEPCHAT_WORKDIR={work_dir}"))
        .arg("-e")
        .arg(format!("PIP_TARGET={site_dir}"))
        .arg("-e")
        .arg(format!("PYTHONPATH={site_dir}"))
        .arg("-e")
        .arg("PIP_DISABLE_PIP_VERSION_CHECK=1")
        .arg("--label")
        .arg(format!("deepchat-container={run_id}"));
    let cache_dir = pip_cache_dir();
    cmd.arg("-v")
        .arg(format!("{cache_dir}:{work_dir}/.cache/pip"));
    if let Some(index_url) = pip_index_url() {
        cmd.arg("-e").arg(format!("PIP_INDEX_URL={index_url}"));
    }
    if let Some(extra_url) = pip_extra_index_url() {
        cmd.arg("-e")
            .arg(format!("PIP_EXTRA_INDEX_URL={extra_url}"));
    }
    match code_exec_network_mode() {
        crate::ui::code_exec_container_env::CodeExecNetwork::None => {
            cmd.arg("--network=none");
        }
        crate::ui::code_exec_container_env::CodeExecNetwork::Host => {
            cmd.arg("--network=host");
        }
        crate::ui::code_exec_container_env::CodeExecNetwork::Bridge => {}
    }
    let output = cmd
        .arg("python:3.11-slim")
        .arg("sleep")
        .arg("infinity")
        .output()
        .map_err(|e| format!("Docker 启动失败：{e}"))?;
    if !output.status.success() {
        return Err(format!(
            "Docker 启动失败：{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        return Err("Docker 启动失败：未返回容器 ID".to_string());
    }
    Ok(id)
}

fn is_container_running(container_id: &str) -> bool {
    if let Ok(output) = Command::new("docker")
        .arg("inspect")
        .arg("-f")
        .arg("{{.State.Running}}")
        .arg(container_id)
        .output()
    {
        if !output.status.success() {
            return false;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        return text.trim() == "true";
    }
    false
}

fn write_code_file(container_id: &str, run_id: &str, code: &str) -> Result<(), String> {
    let run_dir = run_dir();
    let tmp_dir = tmp_dir();
    let site_dir = pip_target_dir();
    let _ = Command::new("docker")
        .arg("exec")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("mkdir -p {run_dir} {tmp_dir} {site_dir}"))
        .status();
    let mut cmd = Command::new("docker");
    cmd.arg("exec")
        .arg("-i")
        .arg(container_id)
        .arg("sh")
        .arg("-lc")
        .arg(format!("cat > {}", code_path(run_id)))
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

fn code_path(run_id: &str) -> String {
    format!("{}/{}.py", run_dir(), run_id)
}
