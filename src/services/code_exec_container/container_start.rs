use crate::services::code_exec_container_env::{
    code_exec_image, code_exec_network_mode, pip_cache_dir, pip_extra_index_url, pip_index_url,
    pip_target_dir, prepare_pip_cache_dir, site_tmpfs_mb, tmp_dir, work_dir,
};
use crate::services::workspace::WorkspaceConfig;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn start_container(workspace: Option<&WorkspaceConfig>) -> Result<String, String> {
    prepare_pip_cache_dir();
    let run_id = new_container_run_id();
    let mut cmd = build_container_command(&run_id);
    add_pip_cache_mount(&mut cmd);
    add_pip_index_envs(&mut cmd);
    add_workspace_mount(&mut cmd, workspace);
    apply_network_mode(&mut cmd);
    run_container_command(cmd)
}

fn new_container_run_id() -> String {
    format!(
        "deepchat-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn build_container_command(run_id: &str) -> Command {
    let mut cmd = Command::new("docker");
    configure_container_command(&mut cmd, run_id);
    cmd
}

fn configure_container_command(cmd: &mut Command, run_id: &str) {
    add_container_base_args(cmd);
    add_container_tmpfs(cmd);
    add_container_envs(cmd);
    cmd.arg("--label")
        .arg(format!("deepchat-container={run_id}"));
}

fn add_container_base_args(cmd: &mut Command) {
    cmd.arg("run")
        .arg("-d")
        .arg("--cpus=1")
        .arg("--memory=512m")
        .arg("--pids-limit=128")
        .arg("--read-only")
        .arg("--user=1000:1000")
        .arg("--cap-drop=ALL")
        .arg("--security-opt=no-new-privileges");
}

fn add_container_tmpfs(cmd: &mut Command) {
    let work_dir = work_dir();
    cmd.arg("--tmpfs").arg(format!(
        "{work_dir}:rw,exec,nosuid,size={}m,uid=1000,gid=1000,mode=755",
        site_tmpfs_mb()
    ));
}

fn add_container_envs(cmd: &mut Command) {
    let work_dir = work_dir();
    let tmp_dir = tmp_dir();
    let site_dir = pip_target_dir();
    let cache_dir = format!("{work_dir}/.cache/pip");
    cmd.arg("-e")
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
        .arg(format!("PIP_CACHE_DIR={cache_dir}"))
        .arg("-e")
        .arg("PIP_DISABLE_PIP_VERSION_CHECK=1");
}

fn add_pip_cache_mount(cmd: &mut Command) {
    let cache_dir = pip_cache_dir();
    let work_dir = work_dir();
    cmd.arg("-v")
        .arg(format!("{cache_dir}:{work_dir}/.cache/pip"));
}

fn add_workspace_mount(cmd: &mut Command, workspace: Option<&WorkspaceConfig>) {
    let Some(workspace) = workspace else {
        return;
    };
    cmd.arg("-v").arg(format!(
        "{}:{}",
        workspace.host_path.display(),
        workspace.mount_path
    ));
    cmd.arg("-e")
        .arg(format!("DEEPCHAT_WORKSPACE={}", workspace.mount_path));
}

fn add_pip_index_envs(cmd: &mut Command) {
    if let Some(index_url) = pip_index_url() {
        cmd.arg("-e").arg(format!("PIP_INDEX_URL={index_url}"));
    }
    if let Some(extra_url) = pip_extra_index_url() {
        cmd.arg("-e")
            .arg(format!("PIP_EXTRA_INDEX_URL={extra_url}"));
    }
}

fn apply_network_mode(cmd: &mut Command) {
    match code_exec_network_mode() {
        crate::services::code_exec_container_env::CodeExecNetwork::None => {
            cmd.arg("--network=none");
        }
        crate::services::code_exec_container_env::CodeExecNetwork::Host => {
            cmd.arg("--network=host");
        }
        crate::services::code_exec_container_env::CodeExecNetwork::Bridge => {}
    }
}

fn run_container_command(mut cmd: Command) -> Result<String, String> {
    let output = cmd
        .arg(code_exec_image())
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

pub(super) fn is_container_running(container_id: &str) -> bool {
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
