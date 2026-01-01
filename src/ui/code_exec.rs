use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct ExecOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub(crate) fn run_python_in_docker(code: &str) -> Result<ExecOutput, String> {
    let mut dir = std::env::temp_dir();
    let uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("时间错误：{e}"))?
        .as_millis();
    dir.push(format!("deepchat_exec_{uniq}"));
    fs::create_dir(&dir).map_err(|e| format!("创建临时目录失败：{e}"))?;
    let code_path = dir.join("code.py");
    fs::write(&code_path, code).map_err(|e| format!("写入临时文件失败：{e}"))?;

    let output = run_docker(&dir);

    let _ = fs::remove_file(&code_path);
    let _ = fs::remove_dir(&dir);
    output
}

fn run_docker(dir_path: &PathBuf) -> Result<ExecOutput, String> {
    let path = dir_path
        .to_str()
        .ok_or_else(|| "临时路径无效".to_string())?;
    let out = Command::new("docker")
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
        .arg("-v")
        .arg(format!("{path}:/work:ro"))
        .arg("python:3.11-slim")
        .arg("python")
        .arg("/work/code.py")
        .output()
        .map_err(|e| format!("Docker 执行失败：{e}"))?;

    Ok(ExecOutput {
        exit_code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    })
}
