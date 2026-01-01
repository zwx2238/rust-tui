use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct ExecOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub(crate) fn run_python_in_docker(code: &str) -> Result<ExecOutput, String> {
    let _uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("时间错误：{e}"))?
        .as_millis();
    run_docker(code)
}

fn run_docker(code: &str) -> Result<ExecOutput, String> {
    let mut child = Command::new("docker")
        .arg("run")
        .arg("--rm")
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
    let out = child
        .wait_with_output()
        .map_err(|e| format!("Docker 执行失败：{e}"))?;

    Ok(ExecOutput {
        exit_code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    })
}
