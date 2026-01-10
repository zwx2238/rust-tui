use crate::args::Args;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) const WORKSPACE_MOUNT: &str = "/workspace";
const WORKSPACE_MAX_BYTES: u64 = 1_000_000;

#[derive(Clone)]
pub(crate) struct WorkspaceConfig {
    pub(crate) host_path: PathBuf,
    pub(crate) mount_path: String,
}

static WORKSPACE_CACHE: OnceLock<Result<WorkspaceConfig, String>> = OnceLock::new();

pub(crate) fn resolve_workspace(args: &Args) -> Result<WorkspaceConfig, String> {
    WORKSPACE_CACHE
        .get_or_init(|| build_workspace_config(args))
        .clone()
}

fn build_workspace_config(args: &Args) -> Result<WorkspaceConfig, String> {
    let path = args.workspace.trim();
    if path.is_empty() {
        return Err("workspace 不能为空".to_string());
    }
    let host_path = Path::new(path)
        .canonicalize()
        .map_err(|e| format!("workspace 路径不可用：{e}"))?;
    if !host_path.is_dir() {
        return Err("workspace 必须是目录".to_string());
    }
    validate_workspace_size(&host_path)?;
    Ok(WorkspaceConfig {
        host_path,
        mount_path: WORKSPACE_MOUNT.to_string(),
    })
}

fn validate_workspace_size(path: &Path) -> Result<(), String> {
    let mut total: u64 = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("workspace 读取失败：{e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("workspace 读取失败：{e}"))?;
            let meta = std::fs::symlink_metadata(entry.path())
                .map_err(|e| format!("workspace 读取失败：{e}"))?;
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                stack.push(entry.path());
                continue;
            }
            if meta.is_file() {
                total = total.saturating_add(meta.len());
                if total > WORKSPACE_MAX_BYTES {
                    return Err(format!("workspace 大小超过限制：{} bytes", total));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn resolve_container_path(
    input: &str,
    workspace: &WorkspaceConfig,
) -> Result<String, String> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err("path 不能为空".to_string());
    }
    let input_path = Path::new(raw);
    if input_path.is_absolute() {
        return resolve_absolute_path(raw, input_path, workspace);
    }
    Ok(resolve_relative_path(raw, workspace))
}

fn resolve_absolute_path(
    raw: &str,
    input_path: &Path,
    workspace: &WorkspaceConfig,
) -> Result<String, String> {
    if raw.starts_with(&workspace.mount_path) {
        return Ok(raw.to_string());
    }
    let canonical = input_path
        .canonicalize()
        .map_err(|e| format!("路径不可用：{e}"))?;
    if !canonical.starts_with(&workspace.host_path) {
        return Err("禁止访问 workspace 之外的路径".to_string());
    }
    let suffix = canonical
        .strip_prefix(&workspace.host_path)
        .map_err(|_| "路径不可用".to_string())?;
    Ok(build_container_path(&workspace.mount_path, suffix))
}

fn resolve_relative_path(raw: &str, workspace: &WorkspaceConfig) -> String {
    build_container_path(&workspace.mount_path, Path::new(raw))
}

fn build_container_path(mount_path: &str, suffix: &Path) -> String {
    let mut out = PathBuf::from(mount_path);
    if !suffix.as_os_str().is_empty() {
        out.push(suffix);
    }
    out.to_string_lossy().to_string()
}
