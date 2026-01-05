use crate::ui::code_exec_container_env::{pip_target_dir, run_dir, tmp_dir};
use std::io::Write;
use std::process::{Command, Stdio};

pub(super) fn write_code_file(container_id: &str, run_id: &str, code: &str) -> Result<(), String> {
    let run_dir = run_dir();
    let tmp_dir = tmp_dir();
    let site_dir = pip_target_dir();
    ensure_container_dirs(container_id, &run_dir, &tmp_dir, &site_dir);
    write_code_via_stdin(container_id, &code_path(run_id), code)
}

pub(super) fn remove_code_file(container_id: &str, run_id: &str) -> Result<(), String> {
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

pub(super) fn write_script_file(
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

pub(super) fn remove_script_file(container_id: &str, run_id: &str, ext: &str) -> Result<(), String> {
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

pub(super) fn code_path(run_id: &str) -> String {
    script_path(run_id, "py")
}

pub(super) fn script_path(run_id: &str, ext: &str) -> String {
    format!("{}/{}.{}", run_dir(), run_id, ext)
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
