use std::io::Write;
use std::process::Command;

use super::ToolResult;
use super::tool_err;

pub(super) fn run_container_python(
    container_id: &str,
    script: &str,
    input: &[u8],
) -> Result<String, ToolResult> {
    let output = run_docker_python(container_id, script, input)?;
    decode_output(output)
}

fn run_docker_python(
    container_id: &str,
    script: &str,
    input: &[u8],
) -> Result<std::process::Output, ToolResult> {
    Command::new("docker")
        .arg("exec")
        .arg("-i")
        .arg(container_id)
        .arg("python")
        .arg("-c")
        .arg(script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input)?;
            }
            child.wait_with_output()
        })
        .map_err(|e| tool_err(format!("Docker 执行失败：{e}")))
}

fn decode_output(output: std::process::Output) -> Result<String, ToolResult> {
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(tool_err(if err.is_empty() {
        "Docker 执行失败".to_string()
    } else {
        err
    }))
}
